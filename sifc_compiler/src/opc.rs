use crate::dreg::DReg;

/// Opc is the representation of opcodes in the sif vm.
#[derive(Clone, Debug)]
pub enum Opc<'o> {
    // Binary ops
    Add {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Sub {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Mul {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Div {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Mod {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Lteq {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Lt {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Gteq {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Gt {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Eq {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Neq {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Lnot {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Land {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Lor {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },
    Lxor {
        src1: &'o DReg,
        src2: &'o DReg,
        dest: &'o DReg,
    },

    // Unary ops
    Lneg {
        src1: &'o DReg,
        dest: &'o DReg,
    },
    Nneg {
        src1: &'o DReg,
        dest: &'o DReg,
    },
}
