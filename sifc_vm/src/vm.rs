use sifc_compiler::{
    dreg::DReg,
    instr::Instr,
    opc::{Op, OpTy},
    sifv::SifVal,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct VM {
    /// Code section. This contains the instructions compiled from
    /// the ast from the compiler and is assumed to be valid.
    code: Vec<Instr>,

    /// Data registers.
    dregs: Vec<Rc<RefCell<DReg>>>,

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
    pub fn new(i: Vec<Instr>, dr: Vec<Rc<RefCell<DReg>>>) -> VM {
        VM {
            code: i,
            dregs: dr,
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
            self.execute();
            self.cdr = self.cdr + 1;
        }
    }

    fn execute(&mut self) {
        let idx = self.cdr;
        let curr = &self.code[idx];
    }
}
