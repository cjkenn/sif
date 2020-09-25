use crate::{dreg::DReg, sifv::SifVal};

/// Opc is the representation of opcodes in the sif vm.
#[derive(Clone, Debug)]
pub enum Opc {
    // Binary ops
    Add { src1: DReg, src2: DReg, dest: DReg },
    Sub { src1: DReg, src2: DReg, dest: DReg },
    Mul { src1: DReg, src2: DReg, dest: DReg },
    Div { src1: DReg, src2: DReg, dest: DReg },
    Mod { src1: DReg, src2: DReg, dest: DReg },
    Lteq { src1: DReg, src2: DReg, dest: DReg },
    Lt { src1: DReg, src2: DReg, dest: DReg },
    Gteq { src1: DReg, src2: DReg, dest: DReg },
    Gt { src1: DReg, src2: DReg, dest: DReg },
    Eq { src1: DReg, src2: DReg, dest: DReg },
    Neq { src1: DReg, src2: DReg, dest: DReg },
    Lnot { src1: DReg, src2: DReg, dest: DReg },
    Land { src1: DReg, src2: DReg, dest: DReg },
    Lor { src1: DReg, src2: DReg, dest: DReg },
    Lxor { src1: DReg, src2: DReg, dest: DReg },

    // Unary ops
    Lneg { src1: DReg, dest: DReg },
    Nneg { src1: DReg, dest: DReg },

    // Load and store ops
    Ldc { dest: DReg, val: SifVal },
}
