use crate::{dreg::DReg, sifv::SifVal};
use std::rc::Rc;

/// Opc is the representation of opcodes in the sif vm.
#[derive(Clone, Debug)]
pub enum Opc {
    // Binary ops
    Add {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Sub {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Mul {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Div {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Mod {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Lteq {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Lt {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Gteq {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Gt {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Eq {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Neq {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Lnot {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Land {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Lor {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Lxor {
        src1: Rc<DReg>,
        src2: Rc<DReg>,
        dest: Rc<DReg>,
    },

    // Unary ops
    Lneg {
        src1: Rc<DReg>,
        dest: Rc<DReg>,
    },
    Nneg {
        src1: Rc<DReg>,
        dest: Rc<DReg>,
    },

    // Load and store ops
    Ldc {
        dest: Rc<DReg>,
        val: SifVal,
    },
}
