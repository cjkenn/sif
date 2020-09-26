use crate::{
    dreg::DReg,
    instr::Instr,
    opc::{Op, OpTy},
    sifv::SifVal,
};
use sifc_err::compile_err::{CompileErr, CompileErrTy};
use sifc_parse::{ast::AstNode, token::TokenTy};
use std::cell::RefCell;
use std::rc::Rc;

pub type CompileResult = Result<Vec<Instr>, CompileErr>;

pub struct Compiler<'c> {
    /// Ast supplied by the parser, assumed to be correct.
    ast: &'c AstNode,

    /// Vector of instructions which the compiler will prepare and fill.
    /// This refers to the code section of the vm layout. It's size should be
    /// known before interpreting begins.
    ops: Vec<Instr>,

    /// Vector of data registers. We expect the list to contain already initialized
    /// data registers with correct names and no values contained within. This is a vec of
    /// pointers to mutable cells, although it's rare they will be mutated: occasionally
    /// constant values needed to be changed inside a register (particularly in load
    /// and store operations).
    dregs: Vec<Rc<RefCell<DReg>>>,

    /// Current number of labels in the block being translated.
    lbl_cnt: usize,

    /// Current available register
    ri: usize,
}

impl<'c> Compiler<'c> {
    pub fn new(a: &'c AstNode, ds: Vec<Rc<RefCell<DReg>>>) -> Compiler<'c> {
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
                TokenTy::Plus => self.binop(OpTy::Add, lhs, rhs),
                TokenTy::Minus => self.binop(OpTy::Sub, lhs, rhs),
                TokenTy::Star => self.binop(OpTy::Mul, lhs, rhs),
                TokenTy::Slash => self.binop(OpTy::Div, lhs, rhs),
                TokenTy::Percent => self.binop(OpTy::Modu, lhs, rhs),
                _ => (),
            },
            AstNode::LogicalExpr { op_tkn, lhs, rhs } => match op_tkn.ty {
                TokenTy::EqEq => self.binop(OpTy::Eq, lhs, rhs),
                TokenTy::LtEq => self.binop(OpTy::LtEq, lhs, rhs),
                TokenTy::Lt => self.binop(OpTy::Lt, lhs, rhs),
                TokenTy::GtEq => self.binop(OpTy::GtEq, lhs, rhs),
                TokenTy::Gt => self.binop(OpTy::Gt, lhs, rhs),
                TokenTy::AmpAmp => self.binop(OpTy::Land, lhs, rhs),
                TokenTy::PipePipe => self.binop(OpTy::Lor, lhs, rhs),
                TokenTy::BangEq => self.binop(OpTy::Lnot, lhs, rhs),
                _ => (),
            },
            AstNode::UnaryExpr { op_tkn, rhs } => match op_tkn.ty {
                TokenTy::Bang => self.unop(OpTy::Lneg, rhs),
                TokenTy::Minus => self.unop(OpTy::Nneg, rhs),
                _ => (),
            },
            _ => (),
        }
    }

    fn binop(&mut self, ty: OpTy, lhs: &AstNode, rhs: &AstNode) {
        let r0 = self.binarg(lhs);
        let r1 = self.binarg(rhs);

        let op = Op::Binary {
            ty: ty,
            src1: Rc::clone(&self.dregs[r0]),
            src2: Rc::clone(&self.dregs[r1]),
            dest: Rc::clone(&self.nextreg()),
        };
        self.push_op(op);
    }

    fn unop(&mut self, ty: OpTy, rhs: &AstNode) {
        let r0 = self.binarg(rhs);
        let op = Op::Unary {
            ty: ty,
            src1: Rc::clone(&self.dregs[r0]),
            dest: Rc::clone(&self.nextreg()),
        };
        self.push_op(op);
    }

    // Returns the index of the register in which the last stored value is
    fn binarg(&mut self, arg: &AstNode) -> usize {
        match arg {
            AstNode::PrimaryExpr { tkn } => match tkn.ty {
                TokenTy::Val(v) => {
                    let sifv = SifVal::Num(v);
                    let d = self.nextreg();
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::Load {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);

                    return self.ri - 1;
                }
                TokenTy::True => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(true);
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::Load {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);

                    return self.ri - 1;
                }
                TokenTy::False => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(false);
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::Load {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);

                    return self.ri - 1;
                }
                _ => {
                    return self.ri - 1;
                }
            },
            _ => {
                self.expr(arg);
                return self.ri - 1;
            }
        }
    }

    fn push_op(&mut self, op: Op) {
        let i = Instr::new(self.currlbl(), op);
        self.ops.push(i);
    }

    fn currlbl(&mut self) -> String {
        format!("lbl{}", self.lbl_cnt)
    }

    fn newlbl(&mut self) {
        self.lbl_cnt = self.lbl_cnt + 1;
    }

    fn nextreg(&mut self) -> Rc<RefCell<DReg>> {
        let reg = Rc::clone(&self.dregs[self.ri]);
        self.ri = self.ri + 1;
        reg
    }

    fn prevreg(&mut self) -> Rc<RefCell<DReg>> {
        if self.ri == 0 {
            return Rc::clone(&self.dregs[self.ri]);
        }

        Rc::clone(&self.dregs[self.ri - 1])
    }
}
