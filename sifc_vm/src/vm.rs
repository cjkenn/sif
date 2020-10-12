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
                OK => {}
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
        let curr = &self.code[idx];

        match &curr.op {
            Op::LoadC { ty: _, dest, val } => {
                let reg = &self.dregs[*dest];
                reg.borrow_mut().cont = Some(val.clone());
            }
            Op::LoadN { ty: _, dest, name } => {
                let reg = &self.dregs[*dest];
                match self.heap.get(name) {
                    Some(n) => {
                        reg.borrow_mut().cont = Some(n.clone());
                    }
                    None => {
                        let err = RuntimeErr::new(RuntimeErrTy::InvalidName(name.clone()));
                        return Err(err);
                    }
                };
            }
            Op::StoreC { ty: _, name, val } => {}
            _ => {}
        };

        Ok(())
    }
}
