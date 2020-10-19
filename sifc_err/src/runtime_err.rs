use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum RuntimeErrTy {
    InvalidName(String),
    InvalidIncr,
    InvalidIncrTy,
    InvalidDecr,
    InvalidDecrTy,
    InvalidOp,
    InvalidJump,
    TyMismatch,
    NotAnArray(String),
    InvalidFnSym(String),
}

#[derive(Debug, Clone)]
pub struct RuntimeErr {
    pub ty: RuntimeErrTy,
    pub line: usize,
}

impl RuntimeErr {
    pub fn new(t: RuntimeErrTy, l: usize) -> RuntimeErr {
        RuntimeErr { ty: t, line: l }
    }
}

impl SifErr for RuntimeErr {
    fn emit(&self) {
        eprintln!(
            "sif: runtime error at instruction #{} : {}",
            self.line,
            self.to_msg()
        );
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
            RuntimeErrTy::InvalidJump => {
                String::from("could not compute jump index from instruction")
            }
            RuntimeErrTy::TyMismatch => {
                String::from("operator cannot be applied to value in desired register")
            }
            RuntimeErrTy::NotAnArray(n) => {
                format!("Cannot load value: '{}' is not an array or sized type", n)
            }
            RuntimeErrTy::InvalidFnSym(s) => {
                format!("Cannot call function {}, declaration not found", s)
            }
        }
    }
}
