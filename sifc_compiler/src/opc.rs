use crate::{dreg::DReg, sifv::SifVal};
use std::cell::RefCell;
use std::rc::Rc;

/// Opc is the representation of opcodes in the sif vm.
#[derive(Clone, Debug)]
pub enum Opc {
    // Binary ops
    Add {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Sub {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Mul {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Div {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Mod {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Lteq {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Lt {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Gteq {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Gt {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Eq {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Neq {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Lnot {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Land {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Lor {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Lxor {
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },

    // Unary ops
    Lneg {
        src1: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Nneg {
        src1: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },

    // Load and store ops
    Ldc {
        dest: Rc<RefCell<DReg>>,
        val: SifVal,
    },
}
