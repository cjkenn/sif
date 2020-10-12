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

    /// Heap section. This contains arrays, tables, records, and globals.
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
                    None => {
                        let err = RuntimeErr::new(RuntimeErrTy::InvalidName(name.clone()));
                        return Err(err);
                    }
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
                // let destval = self.heap.get(destname);
                match self.heap.get(destname) {
                    Some(v) => self.heap.insert(srcname.to_string(), v.clone()),
                    None => self.heap.insert(srcname.to_string(), SifVal::Null),
                };
            }
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
                        _ => {
                            let err = RuntimeErr::new(RuntimeErrTy::InvalidIncrTy);
                            return Err(err);
                        }
                    },
                    None => {
                        let err = RuntimeErr::new(RuntimeErrTy::InvalidIncr);
                        return Err(err);
                    }
                };
            }
            Op::Decrr { ty: _, src } => {
                let reg = &self.dregs[*src];
                let contents = reg.borrow().cont.clone();
                match contents {
                    Some(v) => match v {
                        SifVal::Num(n) => reg.borrow_mut().cont = Some(SifVal::Num(n - 1.0)),
                        _ => {
                            let err = RuntimeErr::new(RuntimeErrTy::InvalidDecrTy);
                            return Err(err);
                        }
                    },
                    None => {
                        let err = RuntimeErr::new(RuntimeErrTy::InvalidDecr);
                        return Err(err);
                    }
                };
            }
            _ => {}
        };

        Ok(())
    }
}
