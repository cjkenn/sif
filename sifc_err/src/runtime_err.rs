use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum RuntimeErrTy {
    InvalidName(String),
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
        }
    }
}
