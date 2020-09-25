use crate::instr::Instr;
use sifc_err::compile_err::CompileErr;
use sifc_parse::ast::AstNode;

pub struct Compiler<'c> {
    ast: &'c AstNode,
}

impl<'c> Compiler<'c> {
    pub fn new(a: &'c AstNode) -> Compiler {
        Compiler { ast: a }
    }

    pub fn compile(&mut self) -> Result<Vec<Instr>, CompileErr> {
        Ok(Vec::new())
    }
}
