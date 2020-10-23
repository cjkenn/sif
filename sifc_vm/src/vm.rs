use crate::dreg::DReg;

use sifc_compiler::{
    instr::Instr,
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};

use sifc_std::Std;

use sifc_err::runtime_err::{RuntimeErr, RuntimeErrTy};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

type DataRegisterVec = Vec<Rc<RefCell<DReg>>>;

// Max size of data register vec. Or rather, right now this
// represents the initial size of the vec, but technically it
// can grow beyond this value.
const DREG_MAX_LEN: usize = 1024;

pub struct VM<'v> {
    /// Code section. This contains the instructions compiled from
    /// the ast from the compiler and is assumed to be valid.
    code: Vec<Instr>,

    /// Declaration section. This contains function declarations that we
    /// jump to from the code section, and then return back to the code section
    /// when completed.
    decls: Vec<Instr>,

    /// Contains all required sections and relevant instructions in one vector. This
    /// is usually built from extending vectors containing other sections.
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

    /// Data register vector.
    dregs: DataRegisterVec,

    /// Special data register that holds values from function returns. This register
    /// is used to place return value before jumping out of a function body, and to
    /// read from in following code after the function has returned.
    frr: Rc<RefCell<DReg>>,

    /// Special control value which holds the instruction number to jump to when
    /// a function returns.
    cdr: usize,

    /// Heap section. This contains arrays, tables, and globals. We use the
    /// name of the data as a key to retrieve and store information in the heap.
    /// This is likely not memory efficient and a more sophisticated structure + allocating
    /// space before vm startup could be more performant.
    heap: HashMap<String, SifVal>,

    /// Standard libary function mappings
    stdlib: Std<'v>,

    /// Stack for storing function params. Note that we do not use this for function return
    /// values, which are placed into frr.
    fnst: Vec<SifVal>,

    /// Index of the start of the code vector. The IP initially points to this
    /// instruction, as this is where execution would normally begin.
    csi: usize,

    /// Current instruction being executed. This is an index into the program vector,
    /// initially set to the start of the code vector.
    ip: usize,

    /// Whether or not to trace execution of the vm. This is dependent on flags passed in
    /// from the driver.
    trace: bool,
}

impl<'v> VM<'v> {
    pub fn new(
        i: Vec<Instr>,
        d: Vec<Instr>,
        p: Vec<Instr>,
        jt: HashMap<usize, usize>,
        ft: HashMap<String, usize>,
        tr: bool,
    ) -> VM<'v> {
        // Init data register array
        let mut regs = Vec::with_capacity(DREG_MAX_LEN);
        for i in 0..DREG_MAX_LEN - 1 {
            let reg = DReg::new(format!("r{}", i));
            regs.push(Rc::new(RefCell::new(reg)));
        }

        let code_start = d.len();

        VM {
            code: i,
            decls: d,
            prog: p,
            fntab: ft,
            jumptab: jt,
            dregs: regs,
            frr: Rc::new(RefCell::new(DReg::new(String::from("frr")))),
            cdr: 0,
            heap: HashMap::new(),
            stdlib: Std::new(),
            fnst: Vec::new(),
            csi: code_start,
            ip: code_start,
            trace: tr,
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeErr> {
        while self.ip < self.prog.len() {
            self.execute()?;
            self.ip = self.ip + 1;
        }
        Ok(())
    }

    pub fn inspect_dreg(&self, idx: usize) -> Option<SifVal> {
        let reg = &self.dregs[idx];
        reg.borrow().cont.clone()
    }

    pub fn inspect_heap(&self, name: String) -> Option<&SifVal> {
        self.heap.get(&name)
    }

    fn execute(&mut self) -> Result<(), RuntimeErr> {
        let idx = self.ip;

        if self.trace {
            self.trace_instr(&self.prog[idx]);
        }

        let curr = &self.prog[idx].op;
        match curr {
            Op::LoadC { dest, val } => self.loadc(*dest, val)?,
            Op::LoadN { dest, name } => self.loadn(*dest, name)?,
            Op::MvFRR { dest } => self.mvfrr(*dest)?,
            Op::Mv { src, dest } => self.mv(*src, *dest)?,
            Op::LoadArrs { name, dest } => self.loadarrs(name, *dest)?,
            Op::LoadArrv {
                name,
                idx_reg,
                dest,
            } => self.loadarrv(name, *idx_reg, *dest)?,
            Op::StoreC { name, val } => {
                self.heap.insert(name.to_string(), val.clone());
            }
            Op::StoreR { name, src } => {
                let reg = &self.dregs[*src];
                let to_store = &reg.borrow().cont;
                match to_store {
                    Some(v) => self.heap.insert(name.to_string(), v.clone()),
                    None => self.heap.insert(name.to_string(), SifVal::Null),
                };
            }
            Op::StoreN { srcname, destname } => {
                match self.heap.get(srcname) {
                    Some(v) => {
                        let to_insert = v.clone();
                        self.heap.insert(destname.to_string(), to_insert)
                    }
                    None => self.heap.insert(destname.to_string(), SifVal::Null),
                };
            }
            Op::StoreFRR { name } => {
                let to_store = &self.frr.borrow().cont;
                match to_store {
                    Some(v) => self.heap.insert(name.to_string(), v.clone()),
                    None => self.heap.insert(name.to_string(), SifVal::Null),
                };
            }
            Op::JumpA { lbl: _, lblidx } => {
                let codeidx = self.jumptab.get(lblidx);
                match codeidx {
                    Some(i) => self.ip = *i - 1,
                    None => return Err(self.newerr(RuntimeErrTy::InvalidJump)),
                };
            }
            Op::JumpCnd {
                kind,
                src,
                lbl: _,
                lblidx,
            } => {
                let reg = &self.dregs[*src];
                let contents = reg.borrow().cont.clone();
                if contents.is_none() {
                    return Err(self.newerr(RuntimeErrTy::TyMismatch));
                }

                match contents.unwrap() {
                    SifVal::Bl(b) => {
                        match *kind {
                            JmpOpKind::Jmpt => {
                                if b {
                                    let codeidx = self.jumptab.get(lblidx);
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
                                    let codeidx = self.jumptab.get(lblidx);
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
            Op::Unary { kind, src1, dest } => self.unop(kind.clone(), *src1, *dest)?,
            Op::Binary {
                kind,
                src1,
                src2,
                dest,
            } => self.binop(kind.clone(), *src1, *src2, *dest)?,
            Op::Incrr { src } => self.incrr(*src)?,
            Op::Decrr { src } => self.decrr(*src)?,
            Op::Fn { name, params: _ } => {
                self.fntab.insert(name.clone(), self.ip);
            }
            Op::FnRetR { src } => {
                // Set the frr to the correct return value
                let srcreg = &self.dregs[*src];
                let src_contents = srcreg.borrow().cont.clone();
                self.frr.borrow_mut().cont = src_contents;

                // Jump back to the value in cdr, which should be set before
                // the function call executes.
                self.ip = self.cdr;
            }
            Op::FnRet => {
                // Nothing to return here, so we just jump back to regular execution.
                self.ip = self.cdr;
            }
            Op::Call {
                name,
                param_count: _,
            } => {
                let maybe_loc = self.fntab.get(name);
                if maybe_loc.is_none() {
                    return Err(self.newerr(RuntimeErrTy::InvalidFnSym(name.to_string())));
                }

                // Get the function location, and save our current location in cdr. This allows
                // the call to return to our correct spot when completed. Then, jump to
                // the location by setting ip to it.
                let loc = maybe_loc.unwrap();
                self.cdr = self.ip;
                self.ip = *loc;
            }
            Op::StdCall { name, param_count } => {
                // pop sifvals off stack up to param count, then
                // look up fn name in lib table and run function with params
                let mut params = Vec::new();
                let mut i = 0;
                while i < *param_count {
                    let v = self.fnst.pop();
                    params.push(v.unwrap());
                    i += 1;
                }
                self.stdlib.call(name, params);
            }
            Op::FnStackPush { src } => {
                let srcreg = &self.dregs[*src];
                let to_push = srcreg.borrow().cont.clone();
                self.fnst.push(to_push.unwrap());
            }
            Op::FnStackPop { dest } => {
                let to_pop = self.fnst.pop();
                let destreg = &self.dregs[*dest];
                destreg.borrow_mut().cont = to_pop;
            }
            Op::TblI { tabname, key, src } => {
                let srcreg = &self.dregs[*src];
                let to_insert = srcreg.borrow().cont.clone();

                match self.heap.get(tabname) {
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
            Op::TblG { tabname, key, dest } => {
                let destreg = &self.dregs[*dest];

                match self.heap.get(tabname) {
                    Some(n) => match n {
                        SifVal::Tab(hm) => {
                            let val = hm.get(key).unwrap();
                            destreg.borrow_mut().cont = Some(val.clone());
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

    fn loadc(&self, dest: usize, val: &SifVal) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[dest];
        reg.borrow_mut().cont = Some(val.clone());
        Ok(())
    }

    fn loadn(&self, dest: usize, name: &String) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[dest];
        match self.heap.get(name) {
            Some(n) => reg.borrow_mut().cont = Some(n.clone()),
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };
        Ok(())
    }

    fn mvfrr(&self, dest: usize) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[dest];
        let contents = &self.frr.borrow().cont;
        reg.borrow_mut().cont = contents.clone();
        Ok(())
    }

    fn mv(&self, src: usize, dest: usize) -> Result<(), RuntimeErr> {
        let srcreg = &self.dregs[src];
        let destreg = &self.dregs[dest];
        let to_move = &srcreg.borrow().cont;
        destreg.borrow_mut().cont = to_move.clone();
        Ok(())
    }

    fn loadarrs(&self, name: &String, dest: usize) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[dest];
        match self.heap.get(name) {
            Some(n) => match n {
                SifVal::Arr(v) => reg.borrow_mut().cont = Some(SifVal::Num(v.len() as f64)),
                _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };
        Ok(())
    }

    fn loadarrv(&self, name: &String, idx_reg: usize, dest: usize) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[dest];
        let idx_sv = &self.dregs[idx_reg].borrow().cont;
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

        match self.heap.get(name) {
            Some(n) => match n {
                SifVal::Arr(v) => reg.borrow_mut().cont = Some(v[to_idx].clone()),
                _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
        };

        Ok(())
    }

    fn unop(&self, kind: UnOpKind, src1: usize, dest: usize) -> Result<(), RuntimeErr> {
        let srcreg = &self.dregs[src1];
        let destreg = &self.dregs[dest];
        let mb_contents = srcreg.borrow().cont.clone();

        if mb_contents.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let contents = mb_contents.unwrap();

        match kind {
            UnOpKind::Lneg => {
                match contents {
                    SifVal::Bl(bl) => destreg.borrow_mut().cont = Some(SifVal::Bl(!bl)),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
            UnOpKind::Nneg => {
                match contents {
                    SifVal::Num(num) => destreg.borrow_mut().cont = Some(SifVal::Num(-num)),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
        };

        Ok(())
    }

    fn binop(
        &self,
        kind: BinOpKind,
        src1: usize,
        src2: usize,
        dest: usize,
    ) -> Result<(), RuntimeErr> {
        let src1reg = &self.dregs[src1];
        let src2reg = &self.dregs[src2];
        let destreg = &self.dregs[dest];

        // TODO: what if src1 and src2 are the same reg? Can we still borrow?
        let mb_contents1 = src1reg.borrow().cont.clone();
        let mb_contents2 = src2reg.borrow().cont.clone();

        if mb_contents1.is_none() {
            return Err(self.newerr(RuntimeErrTy::RegNoContents(self.reg_str(src1))));
        }

        if mb_contents2.is_none() {
            return Err(self.newerr(RuntimeErrTy::RegNoContents(self.reg_str(src2))));
        }

        let contents1 = mb_contents1.unwrap();
        let contents2 = mb_contents2.unwrap();

        match kind {
            BinOpKind::Add => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 + n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Sub => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 - n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Mul => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 * n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Div => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 / n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Modu => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 % n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Eq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 == n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Neq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 != n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::LtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 <= n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 < n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::GtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 >= n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Gt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 > n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Land => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 && b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lnot => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 != b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            BinOpKind::Lor => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 || b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
        };

        Ok(())
    }

    fn incrr(&self, src: usize) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[src];
        let contents = reg.borrow().cont.clone();

        // If contents are None, we have nothing to increment so we set a runtime err.
        // If contents are Some, we must match on the val and ensure the kind of val
        // can be incremented (ie. only a num). If it isn't we err. If it is, we can
        // replace the value with an incremented one.
        match contents {
            Some(v) => match v {
                SifVal::Num(n) => reg.borrow_mut().cont = Some(SifVal::Num(n + 1.0)),
                _ => return Err(self.newerr(RuntimeErrTy::InvalidIncrTy)),
            },
            None => return Err(self.newerr(RuntimeErrTy::InvalidIncr)),
        };

        Ok(())
    }

    fn decrr(&self, src: usize) -> Result<(), RuntimeErr> {
        let reg = &self.dregs[src];
        let contents = reg.borrow().cont.clone();
        match contents {
            Some(v) => match v {
                SifVal::Num(n) => reg.borrow_mut().cont = Some(SifVal::Num(n - 1.0)),
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
