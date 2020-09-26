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
    Ldc,
    Stc,
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
    Load {
        ty: OpTy,
        dest: Rc<RefCell<DReg>>,
        val: SifVal,
    },
    Store {
        ty: OpTy,
        val: SifVal,
    },
}
