use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum CompileErrTy {}

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
        String::from("compiler error")
    }
}
