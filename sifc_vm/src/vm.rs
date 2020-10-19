use crate::dreg::DReg;

use sifc_compiler::{
    instr::Instr,
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};

use sifc_err::{
    err::SifErr,
    runtime_err::{RuntimeErr, RuntimeErrTy},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

type DataRegisterVec = Vec<Rc<RefCell<DReg>>>;

// Max size of data register vec. Or rather, right now this
// represents the initial size of the vec, but technically it
// can grow beyond this value.
const DREG_MAX_LEN: usize = 1024;

pub struct VM {
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

    /// Heap section. This contains arrays, tables, records, and globals. We use the
    /// name of the data as a key to retrieve and store information in the heap.
    /// This is likely not memory efficient and a more sophisticated structure + allocating
    /// space before vm startup could be more performant.
    heap: HashMap<String, SifVal>,

    /// Index of the start of the code vector. The IP initially points to this
    /// instruction, as this is where execution would normally begin.
    csi: usize,

    /// Current instruction being executed. This is an index into the program vector,
    /// initially set to the start of the code vector.
    ip: usize,
}

impl VM {
    pub fn new(
        i: Vec<Instr>,
        d: Vec<Instr>,
        jt: HashMap<usize, usize>,
        ft: HashMap<String, usize>,
    ) -> VM {
        // Init data register array
        let mut regs = Vec::with_capacity(DREG_MAX_LEN);
        for i in 0..DREG_MAX_LEN - 1 {
            let reg = DReg::new(format!("r{}", i));
            regs.push(Rc::new(RefCell::new(reg)));
        }

        // Decls MUST come first to match table indices
        let code_start = d.len();
        let mut prog_vec = d.clone();
        prog_vec.extend(i.iter().cloned());

        VM {
            code: i,
            decls: d,
            prog: prog_vec,
            fntab: ft,
            jumptab: jt,
            dregs: regs,
            frr: Rc::new(RefCell::new(DReg::new(String::from("frr")))),
            cdr: 0,
            heap: HashMap::new(),
            csi: code_start,
            ip: code_start,
        }
    }

    pub fn run(&mut self) {
        // TODO: code and decls must match up: should we combine them? and set ip to
        // start in the middle?
        while self.ip < self.prog.len() {
            match self.execute() {
                Ok(()) => {}
                Err(e) => {
                    e.emit();
                    eprintln!("sif: exiting due to errors");
                    return;
                }
            };

            self.ip = self.ip + 1;
        }
    }

    fn execute(&mut self) -> Result<(), RuntimeErr> {
        let idx = self.ip;
        let curr = &self.prog[idx].op;
        println!("{:#?}", curr);

        match curr {
            Op::LoadC { dest, val } => {
                let reg = &self.dregs[*dest];
                reg.borrow_mut().cont = Some(val.clone());
            }
            Op::LoadN { dest, name } => {
                let reg = &self.dregs[*dest];
                match self.heap.get(name) {
                    Some(n) => reg.borrow_mut().cont = Some(n.clone()),
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
                };
            }
            Op::LoadArrs { name, dest } => {
                let reg = &self.dregs[*dest];
                match self.heap.get(name) {
                    Some(n) => match n {
                        SifVal::Arr(v) => reg.borrow_mut().cont = Some(SifVal::Num(v.len() as f64)),
                        _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
                    },
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
                };
            }
            Op::LoadArrv { name, idx, dest } => {
                let reg = &self.dregs[*dest];
                let idx_sv = &self.dregs[*idx].borrow().cont;
                if idx_sv.is_none() {
                    return Err(self.newerr(RuntimeErrTy::TyMismatch));
                }

                let to_idx = match &idx_sv.as_ref().unwrap() {
                    SifVal::Num(f) => *f as usize,
                    _ => panic!("invalid array index"),
                };

                match self.heap.get(name) {
                    Some(n) => match n {
                        SifVal::Arr(v) => reg.borrow_mut().cont = Some(v[to_idx].clone()),
                        _ => return Err(self.newerr(RuntimeErrTy::NotAnArray(name.clone()))),
                    },
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
                };
            }
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
            Op::JumpA { lbl: _, lblidx } => {
                let codeidx = self.jumptab.get(lblidx);
                match codeidx {
                    Some(i) => self.ip = *i,
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
                                        Some(i) => self.ip = *i,
                                        None => return Err(self.newerr(RuntimeErrTy::InvalidJump)),
                                    };
                                }
                            }
                            JmpOpKind::Jmpf => {
                                if !b {
                                    let codeidx = self.jumptab.get(lblidx);
                                    match codeidx {
                                        Some(i) => self.ip = *i,
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
            Op::Call { name } => {
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
            Op::Nop => {}
            Op::Stop => {
                eprintln!("sif: stop instruction found, halting execution");
                return Ok(());
            }
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

        if mb_contents1.is_none() || mb_contents2.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
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
        RuntimeErr::new(ty, self.ip + 1) // use ip+1 because linenums are 1-indexed
    }
}
