use crate::sifv::SifVal;

#[derive(Clone, Debug)]
pub enum BinOpKind {
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
}

#[derive(Clone, Debug)]
pub enum UnOpKind {
    Lneg,
    Nneg,
}

#[derive(Clone, Debug)]
pub enum JmpOpKind {
    Jmpt,
    Jmpf,
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
/// loads the constant 10 into register r0.
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
    /// Binary operator with 2 register sources
    Binary {
        kind: BinOpKind,
        src1: usize,
        src2: usize,
        dest: usize,
    },

    /// Unary operator with a single register source
    Unary {
        kind: UnOpKind,
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

    /// Moves the frr into the next available register for subsequent expressions
    MvFRR {
        dest: usize,
    },

    /// Moves (copies) the contents of one register to another. Does not erase the
    /// contents of the src register.
    Mv {
        src: usize,
        dest: usize,
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

    /// Store frr (function return register)
    StoreFRR {
        name: String,
    },

    /// Jump conditionally, based on the value in src register
    JumpCnd {
        kind: JmpOpKind,
        src: usize,
        lbl: String,
        lblidx: usize,
    },

    /// Always jump to given lbl
    JumpA {
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

    /// Function declaration
    Fn {
        name: String,
        params: Vec<String>,
    },

    /// Move the return value in src to the frr register, then
    /// jumps to previous code location.
    FnRetR {
        src: usize,
    },

    /// Function call
    Call {
        name: String,
    },

    /// When we have no expression to return, we simply jump to the appropriate
    /// code location.
    FnRet,

    Nop,  // no-op
    Stop, // halt vm execution
}
