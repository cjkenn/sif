use crate::{dreg::DReg, sifv::SifVal};
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Clone, Debug)]
pub enum Op {
    Binary {
        ty: OpTy,
        src1: Rc<RefCell<DReg>>,
        src2: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    Unary {
        ty: OpTy,
        src1: Rc<RefCell<DReg>>,
        dest: Rc<RefCell<DReg>>,
    },
    LoadC {
        ty: OpTy,
        dest: Rc<RefCell<DReg>>,
        val: SifVal,
    },
    LoadN {
        ty: OpTy,
        dest: Rc<RefCell<DReg>>,
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
        src: Rc<RefCell<DReg>>,
    },
    JumpCnd {
        ty: OpTy,
        src: Rc<RefCell<DReg>>,
        lbl: String,
    },
    JumpA {
        ty: OpTy,
        lbl: String,
    },
    Incrr {
        ty: OpTy,
        src: Rc<RefCell<DReg>>,
    },
    Decrr {
        ty: OpTy,
        src: Rc<RefCell<DReg>>,
    },
    Nop {
        ty: OpTy,
    },
}
