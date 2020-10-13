use crate::err::SifErr;

/// RuntimeErrTy is intended to be the most specific error type in sif.
#[derive(Debug, Clone)]
pub enum RuntimeErrTy {
    InvalidName(String),
    InvalidIncr,
    InvalidIncrTy,
    InvalidDecr,
    InvalidDecrTy,
    InvalidOp,
    TyMismatch,
}

#[derive(Debug, Clone)]
pub struct RuntimeErr {
    pub ty: RuntimeErrTy,
}

impl RuntimeErr {
    pub fn new(t: RuntimeErrTy) -> RuntimeErr {
        RuntimeErr { ty: t }
    }
}

impl SifErr for RuntimeErr {
    fn emit(&self) {
        eprintln!("sif: runtime error - {}", self.to_msg());
    }

    fn to_msg(&self) -> String {
        match &self.ty {
            RuntimeErrTy::InvalidName(n) => format!("invalid name '{}' provided: cannot access", n),
            RuntimeErrTy::InvalidIncr => {
                String::from("cannot increment register: no register contents found")
            }
            RuntimeErrTy::InvalidIncrTy => String::from("cannot increment a non-numerical value"),
            RuntimeErrTy::InvalidDecr => {
                String::from("cannot decrement register: no register contents found")
            }
            RuntimeErrTy::InvalidDecrTy => String::from("cannot decrement a non-numerical value"),
            RuntimeErrTy::InvalidOp => String::from("invalid or unknown instruction found"),
            RuntimeErrTy::TyMismatch => {
                String::from("operator cannot be applied to value in desired register")
            }
        }
    }
}
