use sifc_compiler::sifv::SifVal;
use std::{cell::RefCell, rc::Rc};

pub type DataRegisterVec = Vec<Rc<RefCell<DReg>>>;

// Initial size of data register vec. If we exceeed this len,
// we can increase the size of the vec.
const DREG_INITIAL_LEN: usize = 64;

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

pub fn init() -> DataRegisterVec {
    let mut regs = Vec::with_capacity(DREG_INITIAL_LEN);
    for i in 0..DREG_INITIAL_LEN - 1 {
        let reg = DReg::new(format!("r{}", i));
        regs.push(Rc::new(RefCell::new(reg)));
    }
    regs
}

pub fn expand(mut regs: DataRegisterVec) -> DataRegisterVec {
    // For now, just add 64 more regs
    regs.reserve(DREG_INITIAL_LEN);

    let mut start_idx = regs.len() - 1;
    for _i in 0..DREG_INITIAL_LEN - 1 {
        let reg = DReg::new(format!("r{}", start_idx));
        regs.push(Rc::new(RefCell::new(reg)));
        start_idx += 1;
    }

    regs
}
