use crate::{dreg::DReg, instr::Instr, opc::Opc, sifv::SifVal};
use sifc_err::compile_err::{CompileErr, CompileErrTy};
use sifc_parse::{ast::AstNode, token::TokenTy};

pub type CompileResult = Result<Vec<Instr>, CompileErr>;

pub struct Compiler<'c> {
    /// Ast supplied by the parser, assumed to be correct.
    ast: &'c AstNode,

    /// Vector of instructions which the compiler will prepare and fill.
    /// This refers to the code section of the vm layout. It's size should be
    /// known before interpreting begins.
    ops: Vec<Instr>,

    /// Vector of data registers. We expect the list to contain already initialized
    /// data registers with correct names and no values contained within.
    dregs: Vec<DReg>,

    /// Current number of labels in the block being translated.
    lbl_cnt: usize,

    /// Current available register
    ri: usize,
}

impl<'c> Compiler<'c> {
    pub fn new(a: &'c AstNode, ds: Vec<DReg>) -> Compiler<'c> {
        Compiler {
            ast: a,
            ops: Vec::new(),
            dregs: ds,
            lbl_cnt: 0,
            ri: 0,
        }
    }

    pub fn compile(&mut self) -> CompileResult {
        match self.ast {
            AstNode::Program { blocks } => {
                self.blocks(blocks);
                Ok(self.ops.clone())
            }
            _ => Err(CompileErr::new(CompileErrTy::InvalidAst)),
        }
    }

    fn blocks(&mut self, blocks: &Vec<AstNode>) {
        for block in blocks {
            match block {
                AstNode::Block { decls, .. } => self.decls(decls),
                AstNode::ExprStmt { expr } => self.expr(expr),
                _ => (),
            }
        }
    }

    fn decls(&mut self, decls: &Vec<AstNode>) {
        for decl in decls {
            match decl {
                AstNode::ExprStmt { expr } => {
                    self.expr(expr);
                }
                _ => (),
            }
        }
    }

    fn expr(&mut self, expr: &AstNode) {
        match expr {
            AstNode::BinaryExpr { op_tkn, lhs, rhs } => match op_tkn.ty {
                TokenTy::Plus => {
                    self.add(lhs, rhs);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn add(&mut self, lhs: &AstNode, rhs: &AstNode) {
        let r0 = self.binarg(lhs);
        let r1 = self.binarg(rhs);
        let op = Opc::Add {
            src1: self.dregs[r0].clone(),
            src2: self.dregs[r1].clone(),
            dest: self.nextreg().clone(),
        };
        self.push_op(op);
    }

    // Returns the index of the register in which the last stored value is
    fn binarg(&mut self, arg: &AstNode) -> usize {
        match arg {
            AstNode::PrimaryExpr { tkn } => {
                // TODO: type mismatches could occur here (ie. when we read "1" + "2")
                let v = tkn.get_val();
                let sifv = SifVal::Num(v);
                let mut d = self.nextreg().clone();
                d.cont = Some(sifv.clone());
                let op = Opc::Ldc { dest: d, val: sifv };
                self.push_op(op);
                return self.ri - 1;
            }
            _ => {
                self.expr(arg);
                return self.ri - 1;
            }
        }
    }

    fn push_op(&mut self, opc: Opc) {
        let i = Instr::new(self.currlbl(), opc);
        self.ops.push(i);
    }

    fn currlbl(&mut self) -> String {
        format!("lbl{}", self.lbl_cnt)
    }

    fn newlbl(&mut self) {
        self.lbl_cnt = self.lbl_cnt + 1;
    }

    fn nextreg(&mut self) -> &DReg {
        let reg = &self.dregs[self.ri];
        self.ri = self.ri + 1;
        reg
    }

    fn prevreg(&mut self) -> &DReg {
        if self.ri == 0 {
            return &self.dregs[self.ri];
        }

        &self.dregs[self.ri - 1]
    }
}
