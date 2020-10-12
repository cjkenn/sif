use crate::sifv::SifVal;

#[derive(Clone, Debug)]
pub enum OpTy {
    // Binary ops
    Add,
    Sub,
    Mul,
    Div,
    Modu,
    Eq,
    Neq,
    LtEq,
    Lt,
    GtEq,
    Gt,
    Land,
    Lnot,
    Lor,

    // Unary ops
    Lneg,
    Nneg,

    // Load/stores
    Ldc, // load constant
    Ldn, // load name
    Stc, // store constant
    Stn, // store name
    Str, // store register

    // Control flow transfer
    Jmpt, // jump if true
    Jmpf, // jump if false
    Jmp,  // jump always

    // register operations
    Incrr, // increment register contents
    Decrr, // decrement register contents

    Stop, // end program execution
    Nop,  // no op
}

/// Each opcode. Some of these just wrap the op type, but the type is useful for
/// operations: we don't need to have a 1-1 mapping from operators to opcode. We could
/// almost do the same for loads and stores, but the required arguments are not the same.
/// Each opcode contains a type and up to 3 arguments, of the following kinds:
///
/// 1. A register, represented as a usize. This is really just the number of the register
/// 2. A value, for loading or storing
/// 3. A name, for loading from memory
#[derive(Clone, Debug)]
pub enum Op {
    Binary {
        ty: OpTy,
        src1: usize,
        src2: usize,
        dest: usize,
    },

    Unary {
        ty: OpTy,
        src1: usize,
        dest: usize,
    },

    LoadC {
        ty: OpTy,
        dest: usize,
        val: SifVal,
    },

    LoadN {
        ty: OpTy,
        dest: usize,
        name: String,
    },

    StoreC {
        ty: OpTy,
        name: String,
        val: SifVal,
    },

    StoreN {
        ty: OpTy,
        name1: String,
        name2: String,
    },

    StoreR {
        ty: OpTy,
        name: String,
        src: usize,
    },

    JumpCnd {
        ty: OpTy,
        src: usize,
        lbl: String,
    },

    JumpA {
        ty: OpTy,
        lbl: String,
    },

    Incrr {
        ty: OpTy,
        src: usize,
    },

    Decrr {
        ty: OpTy,
        src: usize,
    },

    Nop {
        ty: OpTy,
    },
}
