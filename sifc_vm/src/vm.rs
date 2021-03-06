use crate::{config::VMConfig, dreg::DataRegisterList};
use sifc_bytecode::{
    instr::Instr,
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};
use sifc_err::runtime_err::{RuntimeErr, RuntimeErrTy};
use sifc_std::Std;
use std::collections::HashMap;

pub struct VM<'v> {
    /// Contains all required sections and relevant instructions in one vector. This
    /// is usually built from extending vectors containing other sections.
    /// Sif bytecode currently contains two sections:
    /// 1. The Decl section contains the function declarations contained within the translation
    ///    unit. These are where we jump to from the code section during call instructions,
    ///    and then return back to the code section when completed.
    /// 2. The code section This contains the instructions compiled from
    ///    the ast from the compiler and is assumed to be valid. The start of the code
    ///    section is where program execution begins.
    prog: Vec<Instr>,

    /// Jump table, containing the label indices as keys and the code vector
    /// indices as values. This is generated by the compiler and is assumed
    /// to be correct.
    jumptab: HashMap<usize, usize>,

    /// Second jump table, containing function names as keys and their code vector
    /// indices as values. When we encounter a call instruction, we find the name of
    /// the calling function in this table and jump to the proper instruction index.
    /// This table is populated as the code vector is processed: as each function
    /// declaration is found, we add it to the table. Thus, we cannot call a function
    /// until it has been declared and the declaration must appear before the call in the code
    fntab: HashMap<String, usize>,

    /// Data registers. This is managed by the DataRegisterList struct, which handles getting
    /// references to registers and growing the list when we want more registers.
    dregs: DataRegisterList,

    /// Heap section. This contains arrays, tables, and globals. We use the
    /// name of the data as a key to retrieve and store information in the heap.
    /// This is likely not memory efficient and a more sophisticated structure + allocating
    /// space before vm startup could be more performant.
    heap: HashMap<String, SifVal>,

    /// Standard libary function mappings.
    stdlib: Std<'v>,

    /// Stack for storing function params and return values. We sacrifice a bit of memory efficiency
    /// by not sharing this stacke for function call locations, but this is easier to implement
    /// using SifVals, and it would be annoying to convert every jmp address to a SifVal before
    /// pushing it onto the call stack.
    fn_stack: Vec<SifVal>,

    /// Stack that stores jump locations for calling functions and returning from them. Like a normal
    /// calls stack, when a function is called we push the location to return to on to the stack. Once the
    /// function has finished executing, we pop the location off the call stack and set ip to that
    /// value.
    call_stack: Vec<usize>,

    /// Index of the start of the code vector. The IP initially points to this
    /// instruction, as this is where execution would normally begin.
    csi: usize,

    /// Current instruction being executed. This is an index into the program vector,
    /// initially set to the start of the code vector.
    ip: usize,

    /// Struct containing configuration options for this VM. Options should generally be
    /// passed in from command line flags, and documentation for them should be in the
    /// command line usage/help.
    config: VMConfig,
}

impl<'v> VM<'v> {
    pub fn init(
        full_prog: Vec<Instr>,
        code_start: usize,
        jt: HashMap<usize, usize>,
        ft: HashMap<String, usize>,
        conf: VMConfig,
    ) -> VM<'v> {
        let heap = HashMap::with_capacity(conf.initial_heap_size);
        let reglist = DataRegisterList::init(conf.initial_dreg_count);

        VM {
            prog: full_prog,
            fntab: ft,
            jumptab: jt,
            dregs: reglist,
            heap: heap,
            stdlib: Std::new(),
            fn_stack: Vec::new(),
            call_stack: Vec::new(),
            csi: code_start,
            ip: code_start,
            config: conf,
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeErr> {
        while self.ip < self.prog.len() {
            self.execute()?;
            self.ip = self.ip + 1;
        }
        Ok(())
    }

    pub fn inspect_dreg(&mut self, idx: usize) -> Option<SifVal> {
        let reg = self.dregs.get(idx);
        let contents = reg.borrow().cont.clone();
        contents
    }

    pub fn inspect_heap(&self, name: &str) -> Option<&SifVal> {
        self.heap.get(name)
    }

    fn execute(&mut self) -> Result<(), RuntimeErr> {
        let idx = self.ip;

        if self.config.trace {
            self.trace_instr(&self.prog[idx]);
        }

        let curr = self.prog[idx].op.clone();
        match curr {
            Op::Ldc { dest, val } => self.loadc(dest, val)?,
            Op::Ldn { dest, name } => self.loadn(dest, name)?,
            Op::Mv { src, dest } => self.mv(src, dest)?,
            Op::Ldas { name, dest } => self.loadarrs(name, dest)?,
            Op::Ldav {
                name,
                idx_reg,
                dest,
            } => self.loadarrv(name, idx_reg, dest)?,
            Op::Upda {
                name,
                idx_reg,
                val_reg,
            } => self.newarrv(name, idx_reg, val_reg)?,
            Op::Stc { name, val } => {
                self.heap.insert(name.to_string(), val.clone());
            }
            Op::Str { name, src } => {
                let reg = self.dregs.get(src);
                let to_store = &reg.borrow().cont;
                match to_store {
                    Some(v) => self.heap.insert(name.to_string(), v.clone()),
                    None => self.heap.insert(name.to_string(), SifVal::Null),
                };
            }
            Op::Stn { srcname, destname } => {
                match self.heap.get(&srcname) {
                    Some(v) => {
                        let to_insert = v.clone();
                        self.heap.insert(destname.to_string(), to_insert)
                    }
                    None => self.heap.insert(destname.to_string(), SifVal::Null),
                };
            }
            Op::Jmpa { lblidx } => {
                let codeidx = self.jumptab.get(&lblidx);
                match codeidx {
                    Some(i) => self.ip = *i - 1,
                    None => return Err(self.newerr(RuntimeErrTy::InvalidJump)),
                };
            }
            Op::JmpCnd { kind, src, lblidx } => {
                let reg = self.dregs.get(src);
                let contents = reg.borrow().cont.clone();
                if contents.is_none() {
                    return Err(self.newerr(RuntimeErrTy::TyMismatch));
                }

                match contents.unwrap() {
                    SifVal::Bl(b) => {
                        match kind {
                            JmpOpKind::Jmpt => {
                                if b {
                                    let codeidx = self.jumptab.get(&lblidx);
                                    match codeidx {
                                        // jump to codeidx - 1 as it will be incremented at the end of
                                        // the execution loop
                                        Some(i) => self.ip = *i - 1,
                                        None => return Err(self.newerr(RuntimeErrTy::InvalidJump)),
                                    };
                                }
                            }
                            JmpOpKind::Jmpf => {
                                if !b {
                                    let codeidx = self.jumptab.get(&lblidx);
                                    match codeidx {
                                        // jump to codeidx - 1 as it will be incremented at the end of
                                        // the execution loop
                                        Some(i) => self.ip = *i - 1,
                                        None => return Err(self.newerr(RuntimeErrTy::InvalidJump)),
                                    };
                                }
                            }
                        };
                    }
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
            Op::Unary { kind, src1, dest } => self.unop(kind.clone(), src1, dest)?,
            Op::Binary {
                kind,
                src1,
                src2,
                dest,
            } => self.binop(kind.clone(), src1, src2, dest)?,
            Op::Incrr { src } => self.incrr(src)?,
            Op::Decrr { src } => self.decrr(src)?,
            Op::Fn { name, params: _ } => {
                // This case should never be executed, since fn decls
                // should be in the decls section and not executed in the code loop.
                self.fntab.insert(name.clone(), self.ip);
            }
            Op::FnRet => {
                let ret_loc = self.call_stack.pop();
                if ret_loc.is_none() {
                    return Err(self.newerr(RuntimeErrTy::EmptyCallStack));
                }
                self.ip = ret_loc.unwrap();
            }
            Op::Call {
                name,
                param_count: _,
            } => {
                let maybe_loc = self.fntab.get(&name);
                if maybe_loc.is_none() {
                    return Err(self.newerr(RuntimeErrTy::InvalidFnSym(name.to_string())));
                }

                // Get the function location, and save our current location in cdr. This allows
                // the call to return to our correct spot when completed. Then, jump to
                // the location by setting ip to it.
                let loc = maybe_loc.unwrap();
                self.call_stack.push(self.ip);
                self.ip = *loc;
            }
            Op::StdCall { name, param_count } => {
                // pop sifvals off stack up to param count, then
                // look up fn name in lib table and run function with params
                let mut params = Vec::new();
                let mut i = 0;
                while i < param_count {
                    let v = self.fn_stack.pop();
                    params.push(v.unwrap());
                    i += 1;
                }
                // Std::call will return something always, but if the library function doesn't
                // actually have a return value we will get SifVal::Null
                let result = self.stdlib.call(&name, params);
                self.fn_stack.push(result);
            }
            Op::FnStackPush { src } => {
                let srcreg = self.dregs.get(src);
                let to_push = srcreg.borrow().cont.clone();
                self.fn_stack.push(to_push.unwrap());
            }
            Op::FnStackPop { dest } => {
                let to_pop = self.fn_stack.pop();
                self.dregs.set_contents(dest, to_pop);
            }
            Op::Tbli { tabname, key, src } => {
                let srcreg = self.dregs.get(src);
                let to_insert = srcreg.borrow().cont.clone();

                match self.heap.get(&tabname) {
                    Some(n) => match n {
                        SifVal::Tab(hm) => {
                            let mut map = hm.clone();
                            map.insert(key.to_string(), to_insert.unwrap());
                            self.heap.insert(tabname.to_string(), SifVal::Tab(map));
                        }
                        _ => {}
                    },
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(tabname.clone()))),
                };
            }
            Op::Tblg { tabname, key, dest } => {
                match self.heap.get(&tabname) {
                    Some(n) => match n {
                        SifVal::Tab(hm) => {
                            let val = hm.get(&key).unwrap();
                            self.dregs.set_contents(dest, Some(val.clone()));
                        }
                        _ => {}
                    },
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(tabname.clone()))),
                };
            }
            Op::Stop => {
                eprintln!("sif: stop instruction found, halting execution");
                return Ok(());
            }
            Op::Nop => {}
        };

        Ok(())
    }

    fn loadc(&mut self, dest: usize, val: SifVal) -> Result<(), RuntimeErr> {
        self.dregs.set_contents(dest, Some(val.clone()));
        Ok(())
    }

    fn loadn(&mut self, dest: usize, name: String) -> Result<(), RuntimeErr> {
        match self.heap.get(&name) {
            Some(n) => self.dregs.set_contents(dest, Some(n.clone())),
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };
        Ok(())
    }

    fn mv(&mut self, src: usize, dest: usize) -> Result<(), RuntimeErr> {
        let srcreg = self.dregs.get(src);
        let to_move = &srcreg.borrow().cont;
        self.dregs.set_contents(dest, to_move.clone());
        Ok(())
    }

    fn loadarrs(&mut self, name: String, dest: usize) -> Result<(), RuntimeErr> {
        match self.heap.get(&name) {
            Some(n) => match n {
                SifVal::Arr(v) => self
                    .dregs
                    .set_contents(dest, Some(SifVal::Num(v.len() as f64))),
                _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };
        Ok(())
    }

    fn loadarrv(&mut self, name: String, idx_reg: usize, dest: usize) -> Result<(), RuntimeErr> {
        let idx_reg = self.dregs.get(idx_reg);
        let idx_sv = &idx_reg.borrow().cont;
        if idx_sv.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let maybe_to_idx = match &idx_sv.as_ref().unwrap() {
            SifVal::Num(f) => Some(*f as usize),
            _ => None,
        };

        if maybe_to_idx.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let to_idx = maybe_to_idx.unwrap();

        match self.heap.get(&name) {
            Some(n) => match n {
                SifVal::Arr(v) => self.dregs.set_contents(dest, Some(v[to_idx].clone())),
                _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };

        Ok(())
    }

    fn newarrv(&mut self, name: String, idx_reg: usize, val_reg: usize) -> Result<(), RuntimeErr> {
        let idx_reg = self.dregs.get(idx_reg);
        let idx_sv = &idx_reg.borrow().cont;
        if idx_sv.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let maybe_to_idx = match &idx_sv.as_ref().unwrap() {
            SifVal::Num(f) => Some(*f as usize),
            _ => None,
        };
        if maybe_to_idx.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let to_idx = maybe_to_idx.unwrap();

        let val_reg = self.dregs.get(val_reg);
        let val_sv = &val_reg.borrow().cont;
        if val_sv.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let val_sv_r = val_sv.clone().unwrap();
        let mut new_a: Vec<SifVal>;

        match self.heap.get(&name) {
            Some(n) => match n {
                SifVal::Arr(v) => {
                    if to_idx >= v.len() {
                        return Err(self.newerr(RuntimeErrTy::IndexOutOfBounds(
                            name.clone(),
                            to_idx,
                            v.len(),
                        )));
                    }
                    new_a = v.clone();
                    new_a[to_idx] = val_sv_r;
                }
                _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };

        // This only works because we're guaranteed to return an error early before this name
        // is set if anything is incorrect. If we can ever get here without setting new_a properly,
        // this becomes incorrect.
        self.heap.insert(name, SifVal::Arr(new_a));

        Ok(())
    }

    fn unop(&mut self, kind: UnOpKind, src1: usize, dest: usize) -> Result<(), RuntimeErr> {
        let srcreg = self.dregs.get(src1);
        let mb_contents = srcreg.borrow().cont.clone();

        if mb_contents.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let contents = mb_contents.unwrap();

        match kind {
            UnOpKind::Lneg => {
                match contents {
                    SifVal::Bl(bl) => self.dregs.set_contents(dest, Some(SifVal::Bl(!bl))),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
            UnOpKind::Nneg => {
                match contents {
                    SifVal::Num(num) => self.dregs.set_contents(dest, Some(SifVal::Num(-num))),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
        };

        Ok(())
    }

    fn binop(
        &mut self,
        kind: BinOpKind,
        src1: usize,
        src2: usize,
        dest: usize,
    ) -> Result<(), RuntimeErr> {
        let src1reg = self.dregs.get(src1);
        let src2reg = self.dregs.get(src2);

        // TODO: what if src1 and src2 are the same reg? Can we still borrow?
        let mb_contents1 = src1reg.borrow().cont.clone();
        let mb_contents2 = src2reg.borrow().cont.clone();

        if mb_contents1.is_none() {
            return Err(self.newerr(RuntimeErrTy::RegNoContents(self.reg_str(src1))));
        }

        if mb_contents2.is_none() {
            return Err(self.newerr(RuntimeErrTy::RegNoContents(self.reg_str(src2))));
        }

        let contents1 = mb_contents1.unwrap().clone();
        let contents2 = mb_contents2.unwrap().clone();

        match kind {
            BinOpKind::Add => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Num(n1 + n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Sub => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Num(n1 - n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Mul => match (contents1.clone(), contents2.clone()) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Num(n1 * n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Div => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Num(n1 / n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Modu => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Num(n1 % n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Eq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 == n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Neq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 != n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::LtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 <= n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 < n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::GtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 >= n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Gt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(n1 > n2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Land => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(b1 && b2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lnot => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(b1 != b2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lor => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    self.dregs.set_contents(dest, Some(SifVal::Bl(b1 || b2)));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
        };

        Ok(())
    }

    fn incrr(&mut self, src: usize) -> Result<(), RuntimeErr> {
        let reg = self.dregs.get(src);
        let contents = reg.borrow().cont.clone();

        // If contents are None, we have nothing to increment so we set a runtime err.
        // If contents are Some, we must match on the val and ensure the kind of val
        // can be incremented (ie. only a num). If it isn't we err. If it is, we can
        // replace the value with an incremented one.
        match contents {
            Some(v) => match v {
                SifVal::Num(n) => self.dregs.set_contents(src, Some(SifVal::Num(n + 1.0))),
                _ => return Err(self.newerr(RuntimeErrTy::InvalidIncrTy)),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidIncr)),
        };

        Ok(())
    }

    fn decrr(&mut self, src: usize) -> Result<(), RuntimeErr> {
        let reg = self.dregs.get(src);
        let contents = reg.borrow().cont.clone();
        match contents {
            Some(v) => match v {
                SifVal::Num(n) => self.dregs.set_contents(src, Some(SifVal::Num(n - 1.0))),
                _ => return Err(self.newerr(RuntimeErrTy::InvalidDecrTy)),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidDecr)),
        };

        Ok(())
    }

    fn newerr(&self, ty: RuntimeErrTy) -> RuntimeErr {
        // TODO: instruction count doesnt work sometimes if error is in decl section
        RuntimeErr::new(ty, self.ip + 1 - self.csi)
    }

    fn trace_instr(&self, instr: &Instr) {
        println!("EXEC [code.{}]\t {:#}", instr.line, instr);
    }

    fn reg_str(&self, reg: usize) -> String {
        format!("r{}", reg)
    }
}
