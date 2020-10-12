use crate::{
    instr::Instr,
    opc::{Op, OpTy},
    sifv::SifVal,
};

use sifc_err::compile_err::{CompileErr, CompileErrTy};

use sifc_parse::{
    ast::AstNode,
    symtab::SymTab,
    token::{Token, TokenTy},
};

// TODO: might need to use &Instr instead
pub type CompileResult = Result<Vec<Instr>, CompileErr>;

pub struct Compiler<'c, 's> {
    /// Ast supplied by the parser, assumed to be correct.
    ast: &'c AstNode,

    /// Symbol table, which should be filled and finalized from parsing.
    symtab: &'s SymTab,

    /// Vector of instructions which the compiler will prepare and fill.
    /// This refers to the code section of the vm layout. It's size should be
    /// known before interpreting begins.
    ops: Vec<Instr>,

    /// Current number of labels in the block being translated.
    lblcnt: usize,

    /// Current available register
    ri: usize,
}

impl<'c, 's> Compiler<'c, 's> {
    pub fn new(a: &'c AstNode, st: &'s SymTab) -> Compiler<'c, 's> {
        Compiler {
            ast: a,
            symtab: st,
            ops: Vec::new(),
            lblcnt: 0,
            ri: 0,
        }
    }

    pub fn compile(&mut self) -> CompileResult {
        match self.ast {
            AstNode::Program { blocks } => {
                self.blocks(blocks.to_vec());

                // TODO: we need this for labeling purposes right now, but
                // could probably remove it later
                self.newlbl();
                self.push_op(Op::Nop { ty: OpTy::Nop });

                Ok(self.ops.clone())
            }
            _ => Err(CompileErr::new(CompileErrTy::InvalidAst)),
        }
    }

    pub fn blocks(&mut self, blocks: Vec<AstNode>) {
        for block in blocks {
            self.block(&block);
        }
    }

    pub fn block(&mut self, block: &AstNode) {
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
            } => self.ifstmt(
                cond_expr,
                if_stmts,
                elif_exprs.to_vec(),
                else_stmts.to_vec(),
            ),
            AstNode::ForStmt {
                var_list,
                in_expr_list,
                stmts,
            } => self.forstmt(var_list, in_expr_list, stmts),
            AstNode::ArrayDecl {
                ident_tkn, body, ..
            } => self.arraydecl(ident_tkn, body),
            _ => {
                // generate nothing if we find some unknown block
            }
        }
    }

    pub fn lblcnt(&self) -> usize {
        self.lblcnt
    }

    pub fn newlbl(&mut self) {
        self.lblcnt = self.lblcnt + 1;
    }

    pub fn buildlbl(&self, cnt: usize) -> String {
        format!("lbl{}", cnt)
    }

    pub fn currlbl(&self) -> String {
        format!("lbl{}", self.lblcnt)
    }

    pub fn nextreg(&mut self) -> usize {
        let reg = self.ri;
        self.ri = self.ri + 1;
        reg
    }

    pub fn prevreg(&mut self) -> usize {
        if self.ri == 0 {
            return 0;
        }

        self.ri - 1
    }

    pub fn build_name(&mut self, name: String) -> String {
        format!("ident:{}", name)
    }

    pub fn push_op(&mut self, op: Op) {
        let i = Instr::new(self.currlbl(), op);
        self.ops.push(i);
    }

    pub fn expr(&mut self, expr: &AstNode) {
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
            src1: r0,
            src2: r1,
            dest: self.nextreg(),
        };
        self.push_op(op);
    }

    fn unop(&mut self, ty: OpTy, rhs: &AstNode) {
        let r0 = self.binarg(rhs);
        let op = Op::Unary {
            ty: ty,
            src1: r0,
            dest: self.nextreg(),
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

    fn decls(&mut self, decls: &Vec<AstNode>) {
        for decl in decls {
            match decl {
                AstNode::ExprStmt { expr } => {
                    self.expr(expr);
                }
                AstNode::VarDecl {
                    ident_tkn,
                    is_global: _,
                    lhs,
                } => self.vardecl(ident_tkn, lhs.clone()),
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

    pub fn assign(&mut self, st_name: String, rhs: &AstNode) {
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
                    src: self.prevreg(),
                };
                self.push_op(op);
            }
        };
    }

    // fn upd_op_at_idx(&mut self, op: Op, idx: usize) {
    //     let i = &mut self.ops[idx];
    //     i.op = op;
    // }
}
