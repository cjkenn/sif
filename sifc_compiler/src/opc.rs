use crate::sifv::SifVal;

/// OpTy is only used for opcodes that are structurally the same, but only differ in the
/// resulting operation performed. For example, binary opcodes can be differentiated based
/// on their OpTy fields, but load/store opcodes don't bother with OpTy because they are
/// are structually unique and this would map 1-1 with OpTy, making the type useless.
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

    // Control flow transfer
    Jmpt, // jump if true
    Jmpf, // jump if false
    Jmp,  // jump always
}

/// Each opcode contains a type and up to 3 arguments, of the following kinds:
///
/// 1. A register, represented as a usize. This is really just the number of the register
/// 2. A value, for loading or storing
/// 3. A name, for loading from memory
///
/// The destination location (register or memory) is always the last argument in the op.
/// Examples:
///
/// add r0 r1 r2
/// adds registers r0 and r1, stores in r2. The destination register can overwrite a src register
///
/// ldc 10 r0
/// loads the constent 10 into register r0.
///
/// stc 10 y
/// stores the constant 10 into the memory address where the name y is stored.
///
/// stn x y
/// stores the value located at "x" into the address for "y"
///
/// str r2 y
/// stores the value located in r2 into the address for "y"
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

    /// Load a constant SifVal
    LoadC {
        dest: usize,
        val: SifVal,
    },

    /// Load a name
    LoadN {
        dest: usize,
        name: String,
    },

    /// Loads the size (length) of the array by the given name.
    LoadArrs {
        name: String,
        dest: usize,
    },

    /// Loads the value of the array at the index given. The idx field
    /// is expected to be a register.
    LoadArrv {
        name: String,
        idx: usize,
        dest: usize,
    },

    /// Store a constant
    StoreC {
        val: SifVal,
        name: String,
    },

    /// Store a name
    StoreN {
        srcname: String,
        destname: String,
    },

    /// Store a register
    StoreR {
        src: usize,
        name: String,
    },

    JumpCnd {
        ty: OpTy,
        src: usize,
        lbl: String,
        lblidx: usize,
    },

    JumpA {
        ty: OpTy,
        lbl: String,
        lblidx: usize,
    },

    /// Increment src register
    Incrr {
        src: usize,
    },

    /// Decrement src register
    Decrr {
        src: usize,
    },

    Nop,  // no-op
    Stop, // halt vm execution
}
