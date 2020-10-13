use sifc_compiler::{
    dreg::DReg,
    instr::Instr,
    opc::{Op, OpTy},
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

    /// Data register vector.
    dregs: DataRegisterVec,

    /// Heap section. This contains arrays, tables, records, and globals. We use the
    /// name of the data as a key to retrieve and store information in the heap.
    /// This is likely not memory efficient and a more sophisticated structure + allocating
    /// space before vm startup could be more performant.
    heap: HashMap<String, SifVal>,

    /// The call stack.
    callst: Vec<SifVal>,

    /// The data stack.
    datast: Vec<SifVal>,

    /// Current instruction being executed. This is an index into the code vector.
    cdr: usize,

    /// Call stack base index.
    csb: usize,

    /// Data stack base index.
    dsb: usize,

    /// Call stack top index.
    ctop: usize,

    /// Data stack top index.
    dtop: usize,
}

impl VM {
    pub fn new(i: Vec<Instr>) -> VM {
        // Init data register array
        let mut regs = Vec::with_capacity(DREG_MAX_LEN);
        for i in 0..DREG_MAX_LEN - 1 {
            let reg = DReg::new(format!("r{}", i));
            regs.push(Rc::new(RefCell::new(reg)));
        }

        VM {
            code: i,
            dregs: regs,
            heap: HashMap::new(),
            callst: Vec::new(),
            datast: Vec::new(),
            cdr: 0,
            csb: 0,
            dsb: 0,
            ctop: 0,
            dtop: 0,
        }
    }

    pub fn run(&mut self) {
        while self.cdr < self.code.len() {
            match self.execute() {
                Ok(()) => {}
                Err(e) => {
                    e.emit();
                    eprintln!("sif: exiting due to errors");
                    return;
                }
            };

            self.cdr = self.cdr + 1;
        }
    }

    fn execute(&mut self) -> Result<(), RuntimeErr> {
        let idx = self.cdr;
        let curr = &self.code[idx].op;

        match curr {
            Op::LoadC { ty: _, dest, val } => {
                let reg = &self.dregs[*dest];
                reg.borrow_mut().cont = Some(val.clone());
            }
            Op::LoadN { ty: _, dest, name } => {
                let reg = &self.dregs[*dest];
                match self.heap.get(name) {
                    Some(n) => reg.borrow_mut().cont = Some(n.clone()),
                    None => return Err(self.newerr(RuntimeErrTy::InvalidName(name.clone()))),
                };
            }
            Op::StoreC { ty: _, name, val } => {
                self.heap.insert(name.to_string(), val.clone());
            }
            Op::StoreR { ty: _, name, src } => {
                let reg = &self.dregs[*src];
                let to_store = &reg.borrow().cont;
                match to_store {
                    Some(v) => self.heap.insert(name.to_string(), v.clone()),
                    None => self.heap.insert(name.to_string(), SifVal::Null),
                };
            }
            Op::StoreN {
                ty: _,
                srcname,
                destname,
            } => {
                match self.heap.get(destname) {
                    Some(v) => {
                        let to_insert = v.clone();
                        self.heap.insert(srcname.to_string(), to_insert)
                    }
                    None => self.heap.insert(srcname.to_string(), SifVal::Null),
                };
            }
            Op::Unary { ty, src1, dest } => self.unop(ty.clone(), *src1, *dest)?,
            Op::Binary {
                ty,
                src1,
                src2,
                dest,
            } => self.binop(ty.clone(), *src1, *src2, *dest)?,
            Op::Incrr { ty: _, src } => {
                let reg = &self.dregs[*src];
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
            }
            Op::Decrr { ty: _, src } => {
                let reg = &self.dregs[*src];
                let contents = reg.borrow().cont.clone();
                match contents {
                    Some(v) => match v {
                        SifVal::Num(n) => reg.borrow_mut().cont = Some(SifVal::Num(n - 1.0)),
                        _ => return Err(self.newerr(RuntimeErrTy::InvalidDecrTy)),
                    },
                    None => return Err(self.newerr(RuntimeErrTy::InvalidDecr)),
                };
            }
            Op::Nop => {}
            Op::Stop => {
                eprintln!("sif: stop instruction found, halting execution");
                return Ok(());
            }
            _ => return Err(self.newerr(RuntimeErrTy::InvalidOp)),
        };

        Ok(())
    }

    fn unop(&self, ty: OpTy, src1: usize, dest: usize) -> Result<(), RuntimeErr> {
        let srcreg = &self.dregs[src1];
        let destreg = &self.dregs[dest];
        let mb_contents = srcreg.borrow().cont.clone();

        if mb_contents.is_none() {
            return Err(self.newerr(RuntimeErrTy::TyMismatch));
        }

        let contents = mb_contents.unwrap();

        match ty {
            OpTy::Lneg => {
                match contents {
                    SifVal::Bl(bl) => destreg.borrow_mut().cont = Some(SifVal::Bl(!bl)),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
            OpTy::Nneg => {
                match contents {
                    SifVal::Num(num) => destreg.borrow_mut().cont = Some(SifVal::Num(-num)),
                    _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
                };
            }
            _ => return Err(self.newerr(RuntimeErrTy::InvalidOp)),
        };

        Ok(())
    }

    fn binop(&self, ty: OpTy, src1: usize, src2: usize, dest: usize) -> Result<(), RuntimeErr> {
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

        match ty {
            OpTy::Add => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 + n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Sub => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 - n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Mul => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 * n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Div => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 / n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Modu => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Num(n1 % n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Eq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 == n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Neq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 != n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::LtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 <= n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Lt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 < n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::GtEq => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 >= n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Gt => match (contents1, contents2) {
                (SifVal::Num(n1), SifVal::Num(n2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(n1 > n2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Land => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 && b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Lnot => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 != b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            OpTy::Lor => match (contents1, contents2) {
                (SifVal::Bl(b1), SifVal::Bl(b2)) => {
                    destreg.borrow_mut().cont = Some(SifVal::Bl(b1 || b2));
                }
                _ => return Err(self.newerr(RuntimeErrTy::TyMismatch)),
            },
            _ => return Err(self.newerr(RuntimeErrTy::InvalidOp)),
        };

        Ok(())
    }

    fn newerr(&self, ty: RuntimeErrTy) -> RuntimeErr {
        // TODO: encode line information here using the cdr for better errors
        RuntimeErr::new(ty)
    }
}
