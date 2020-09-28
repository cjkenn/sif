use crate::{
    dreg::DReg,
    instr::Instr,
    opc::{Op, OpTy},
    sifv::SifVal,
};
use sifc_err::compile_err::{CompileErr, CompileErrTy};
use sifc_parse::{
    ast::AstNode,
    token::{Token, TokenTy},
};
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
            self.block(block);
        }
    }

    fn block(&mut self, block: &AstNode) {
        match block {
            AstNode::Block { decls, .. } => self.decls(decls),
            AstNode::ExprStmt { expr } => self.expr(expr),
            AstNode::VarDecl {
                ident_tkn,
                is_global: _,
                lhs,
            } => self.vardecl(ident_tkn, lhs.clone()),
            AstNode::IfStmt {
                cond_expr,
                if_stmts,
                elif_exprs,
                else_stmts,
            } => {
                // Generate condition expression
                self.expr(cond_expr);
                self.newlbl();

                // Generate jump. we use nextlbl() and the converse of the
                // condition expression.
                let jmp_op = Op::JumpCnd {
                    ty: OpTy::Jmpf,
                    src: self.prevreg(),
                    lbl: self.nextlbl(),
                };
                self.push_op(jmp_op);

                // Generate statements for when the condition expression is true
                self.block(if_stmts);
                self.newlbl();

                // Generate statements for elif nodes
                for ee in elif_exprs {
                    self.elif(ee);
                }

                // Generate statements for when the condition is false
                self.blocks(else_stmts);
                self.newlbl();
            }
            _ => {
                // generate nothing if we find some unknown block
            }
        }
    }

    fn elif(&mut self, elif: &AstNode) {
        match elif {
            AstNode::ElifStmt { cond_expr, stmts } => {
                self.expr(cond_expr);
                self.newlbl();

                // Generate jump. we use nextlbl() and the converse of the
                // condition expression.
                let jmp_op = Op::JumpCnd {
                    ty: OpTy::Jmpf,
                    src: self.prevreg(),
                    lbl: self.nextlbl(),
                };
                self.push_op(jmp_op);

                self.block(stmts);
            }
            _ => {}
        };
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

    fn vardecl(&mut self, tkn: &Token, rhs: Option<Box<AstNode>>) {
        let st_name = self.build_name(tkn.get_name());
        if rhs.is_none() {
            // We generate a store for an empty value here, to ensure that the name is present
            // in memory if we try to assign to it later. We can detect null value accesses at some
            // point if we want to, or we can leave it to runtime.
            let op = Op::StoreC {
                ty: OpTy::Stc,
                name: st_name,
                val: SifVal::Null,
            };
            self.push_op(op);
            return;
        }

        self.assign(st_name, &rhs.unwrap());
    }

    fn assign(&mut self, st_name: String, rhs: &AstNode) {
        match rhs {
            AstNode::PrimaryExpr { tkn } => {
                match &tkn.ty {
                    TokenTy::Val(v) => {
                        let op = Op::StoreC {
                            ty: OpTy::Stc,
                            name: st_name,
                            val: SifVal::Num(*v),
                        };
                        self.push_op(op);
                    }
                    TokenTy::Str(s) => {
                        let op = Op::StoreC {
                            ty: OpTy::Stc,
                            name: st_name,
                            val: SifVal::Str(s.clone()),
                        };
                        self.push_op(op);
                    }
                    TokenTy::Ident(i) => {
                        let op = Op::StoreN {
                            ty: OpTy::Stn,
                            name1: st_name,
                            name2: self.build_name(i.clone()),
                        };
                        self.push_op(op);
                    }
                    _ => {}
                };
            }
            _ => {
                // We assume that if we aren't assigning a declaration to a constant, we are using an
                // expression. We store based on the correct register from the expression.
                self.expr(&rhs);
                let op = Op::StoreR {
                    ty: OpTy::Str,
                    name: st_name,
                    src: Rc::clone(&self.prevreg()),
                };
                self.push_op(op);
            }
        };
    }

    fn expr(&mut self, expr: &AstNode) {
        match expr {
            AstNode::BinaryExpr { op_tkn, lhs, rhs } => match op_tkn.ty {
                TokenTy::Plus => self.binop(OpTy::Add, lhs, rhs),
                TokenTy::Minus => self.binop(OpTy::Sub, lhs, rhs),
                TokenTy::Star => self.binop(OpTy::Mul, lhs, rhs),
                TokenTy::Slash => self.binop(OpTy::Div, lhs, rhs),
                TokenTy::Percent => self.binop(OpTy::Modu, lhs, rhs),
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
            AstNode::VarAssignExpr {
                ident_tkn,
                is_global: _,
                rhs,
            } => {
                let st_name = self.build_name(ident_tkn.get_name());
                self.assign(st_name, rhs);
            }
            AstNode::PrimaryExpr { .. } => {
                // PrimaryExpr by itself does not generate anything
            }
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
            AstNode::PrimaryExpr { tkn } => match &tkn.ty {
                TokenTy::Val(v) => {
                    let sifv = SifVal::Num(*v);
                    let d = self.nextreg();
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::LoadC {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);
                }
                TokenTy::True => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(true);
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::LoadC {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);
                }
                TokenTy::False => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(false);
                    d.borrow_mut().cont = Some(sifv.clone());

                    let op = Op::LoadC {
                        ty: OpTy::Ldc,
                        dest: d,
                        val: sifv,
                    };
                    self.push_op(op);
                }
                TokenTy::Ident(i) => {
                    let d = self.nextreg();

                    let op = Op::LoadN {
                        ty: OpTy::Ldn,
                        dest: d,
                        name: i.clone(),
                    };
                    self.push_op(op);
                }
                _ => {}
            },
            _ => {
                self.expr(arg);
            }
        };
        self.ri - 1
    }

    fn push_op(&mut self, op: Op) {
        let i = Instr::new(self.currlbl(), op);
        self.ops.push(i);
    }

    fn currlbl(&mut self) -> String {
        format!("lbl{}", self.lbl_cnt)
    }

    fn nextlbl(&mut self) -> String {
        format!("lbl{}", self.lbl_cnt + 1)
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

    fn currreg(&self) -> Rc<RefCell<DReg>> {
        Rc::clone(&self.dregs[self.ri])
    }

    fn advance(&mut self) {
        self.ri = self.ri + 1;
    }

    fn build_name(&mut self, name: String) -> String {
        format!("name::{}", name)
    }
}
