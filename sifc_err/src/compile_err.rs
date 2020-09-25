use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum CompileErrTy {
    InvalidAst,
}

#[derive(Debug, Clone)]
pub struct CompileErr {
    pub ty: CompileErrTy,
}

impl CompileErr {
    pub fn new(t: CompileErrTy) -> CompileErr {
        CompileErr { ty: t }
    }
}

impl SifErr for CompileErr {
    fn emit(&self) {
        println!("sif: Compile error - {}", self.to_msg());
    }

    fn to_msg(&self) -> String {
        match self.ty {
            CompileErrTy::InvalidAst => {
                String::from("fatal: invalid or unknown ast format provided")
            }
        }
    }
}
