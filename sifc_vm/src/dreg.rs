use sifc_compiler::sifv::SifVal;
use std::{cell::RefCell, rc::Rc};

type DRegVec = Vec<Rc<RefCell<DReg>>>;

/// DReg represents a data register.
#[derive(Clone, Debug)]
pub struct DReg {
    pub name: String,
    pub cont: Option<SifVal>,
}

impl DReg {
    pub fn new(n: String) -> DReg {
        DReg {
            name: n,
            cont: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataRegisterList {
    initial_size: usize,
    alloc_count: usize,
    dregs: DRegVec,
}

impl DataRegisterList {
    pub fn init(initial_size: usize) -> DataRegisterList {
        let mut regs = Vec::with_capacity(initial_size);

        for i in 0..initial_size - 1 {
            let reg = DReg::new(format!("r{}", i));
            regs.push(Rc::new(RefCell::new(reg)));
        }

        DataRegisterList {
            initial_size: initial_size,
            alloc_count: 1,
            dregs: regs,
        }
    }

    pub fn get(&mut self, index: usize) -> Rc<RefCell<DReg>> {
        if index >= self.dregs.len() {
            self.expand();
        }
        Rc::clone(&self.dregs[index])
    }

    pub fn set_contents(&self, index: usize, val: Option<SifVal>) {
        self.dregs[index].borrow_mut().cont = val;
    }

    fn expand(&mut self) {
        let grow_size = self.initial_size * self.alloc_count;
        self.dregs.reserve(grow_size);

        let mut start_idx = self.dregs.len() - 1;
        for _i in 0..grow_size - 1 {
            let reg = DReg::new(format!("r{}", start_idx));
            self.dregs.push(Rc::new(RefCell::new(reg)));
            start_idx += 1;
        }
        self.alloc_count += 1;
    }
}
