use crate::{
    instr::Instr,
    opc::{BinOpKind, Op, UnOpKind},
    sifv::SifVal,
};

use sifc_err::compile_err::{CompileErr, CompileErrTy};

use sifc_parse::{
    ast::AstNode,
    token::{Token, TokenTy},
};

use std::collections::HashMap;

/// CompileResult contains information generated by the compiler after
/// compilation is completed.
// TODO: might need to use &Instr instead
pub struct CompileResult {
    /// Vector containing the sequential, linear 3 address code for
    /// a vm to execute.
    pub code: Vec<Instr>,

    /// Vector containing declarations of global vars and function symbols.
    /// A vm can execute this code, but it must be kept separate from the
    /// code vector as it is executed on demand (ie. by call instructions),
    /// not in the order code appears.
    pub decls: Vec<Instr>,

    /// Combination of code and decl section. To build this, we append all the code
    /// instruction at the end of the decl vector. Note that this should be built and
    /// passed in to the jump/fn tables computation to keep the instruction indices
    /// the same.
    pub program: Vec<Instr>,

    /// Jump table that maps label indices to code indices. When you look
    /// up a label index, this returns the index of the first instruction
    /// under than label. Both indices being at 0. For example:
    ///
    /// lbl0:
    ///     stc 10 r0
    ///     stc 11 r1
    /// lbl1:
    ///     add r0 r1 r2
    ///
    /// If we call jumptab.get(0), it would return 0. If we call jumptab.get(1),
    /// we would get back 2, since the third (index 2) instruction is the first under
    /// lbl1.
    /// This requires a separate pass to compute, after the initial compilation phase
    /// which generates instructions and labels.
    pub jumptab: HashMap<usize, usize>,

    /// Similar to jumptab, this contains function names as keys and decl vector
    /// indices as values. When we find a call instruction, we look up the name
    /// being called and find the index to the correct instruction to jump to.
    pub fntab: HashMap<String, usize>,

    /// The index into the program vector indicating the start of the code section. This
    /// can be used by a vm to determine where to start program execution.
    pub code_start: usize,

    /// Any errors that were encountered during compilation. The compiler does not
    /// attempt to continue on any error, it should exit immediately upon error.
    pub err: Option<CompileErr>,
}

pub struct Compiler<'c> {
    /// Ast supplied by the parser, assumed to be correct.
    ast: &'c AstNode,

    /// Vector of instructions which the compiler will prepare and fill.
    /// This refers to the code section of the vm layout. It's size should be
    /// known before interpreting begins.
    ops: Vec<Instr>,

    /// Vector of instructions which the compiler will fill with declarations,
    /// particularly function bodies.
    decls: Vec<Instr>,

    /// Current number of labels in the block being translated.
    lblcnt: usize,

    /// Current available register
    ri: usize,

    /// If true, we write to decl section. If false, write to code section.
    // This is really more of a hack, we should process things without having to
    // switch this flag on and off for decls...
    decl_scope: bool,
}

impl<'c> Compiler<'c> {
    pub fn new(a: &'c AstNode) -> Compiler<'c> {
        Compiler {
            ast: a,
            ops: Vec::new(),
            decls: Vec::new(),
            lblcnt: 0,
            ri: 0,
            decl_scope: false,
        }
    }

    pub fn compile(&mut self) -> CompileResult {
        let mut currerr = None;

        match self.ast {
            AstNode::Program { blocks } => {
                self.blocks(blocks.to_vec());

                // TODO: we need this for labeling purposes right now, but
                // could probably remove it later
                self.newlbl();
                self.push_op(Op::Nop);
            }
            _ => currerr = Some(CompileErr::new(CompileErrTy::InvalidAst)),
        };

        let mut prog_vec = self.decls.clone();
        prog_vec.extend(self.ops.iter().cloned());

        let (jumptab, fntab) = crate::tables::compute(&prog_vec);

        CompileResult {
            code: self.ops.to_vec(),
            decls: self.decls.to_vec(),
            program: prog_vec,
            jumptab: jumptab,
            fntab: fntab,
            code_start: self.decls.len(),
            err: currerr,
        }
    }

    pub fn blocks(&mut self, blocks: Vec<AstNode>) {
        for block in blocks {
            self.block(&block);
        }
    }

    pub fn block(&mut self, block: &AstNode) {
        match block {
            AstNode::Block { decls, .. } => self.blocks(decls.to_vec()),
            AstNode::ExprStmt { expr } => self.expr(expr),
            AstNode::VarDecl {
                ident_tkn,
                is_global: _,
                rhs,
            } => self.vardecl(ident_tkn, rhs.clone()),
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
            AstNode::FnDecl {
                ident_tkn,
                fn_params,
                fn_body,
                ..
            } => self.fndecl(ident_tkn, fn_params, fn_body),
            AstNode::ReturnStmt { ret_expr } => self.ret(ret_expr),
            _ => {
                // generate nothing if we find some unknown block
                // TODO: eventually error here
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

    pub fn push_op(&mut self, op: Op) {
        if self.decl_scope {
            let i = Instr::new(self.lblcnt, op, self.decls.len() + 1);
            self.decls.push(i);
        } else {
            let i = Instr::new(self.lblcnt, op, self.ops.len() + 1);
            self.ops.push(i);
        }
    }

    pub fn expr(&mut self, expr: &AstNode) {
        match expr {
            AstNode::BinaryExpr { op_tkn, lhs, rhs } => match op_tkn.ty {
                TokenTy::Plus => self.binop(BinOpKind::Add, lhs, rhs),
                TokenTy::Minus => self.binop(BinOpKind::Sub, lhs, rhs),
                TokenTy::Star => self.binop(BinOpKind::Mul, lhs, rhs),
                TokenTy::Slash => self.binop(BinOpKind::Div, lhs, rhs),
                TokenTy::Percent => self.binop(BinOpKind::Modu, lhs, rhs),
                TokenTy::EqEq => self.binop(BinOpKind::Eq, lhs, rhs),
                TokenTy::LtEq => self.binop(BinOpKind::LtEq, lhs, rhs),
                TokenTy::Lt => self.binop(BinOpKind::Lt, lhs, rhs),
                TokenTy::GtEq => self.binop(BinOpKind::GtEq, lhs, rhs),
                TokenTy::Gt => self.binop(BinOpKind::Gt, lhs, rhs),
                TokenTy::AmpAmp => self.binop(BinOpKind::Land, lhs, rhs),
                TokenTy::PipePipe => self.binop(BinOpKind::Lor, lhs, rhs),
                TokenTy::BangEq => self.binop(BinOpKind::Lnot, lhs, rhs),
                _ => (),
            },
            AstNode::UnaryExpr { op_tkn, rhs } => match op_tkn.ty {
                TokenTy::Bang => self.unop(UnOpKind::Lneg, rhs),
                TokenTy::Minus => self.unop(UnOpKind::Nneg, rhs),
                _ => (),
            },
            AstNode::VarAssignExpr {
                ident_tkn,
                is_global: _,
                rhs,
            } => {
                let st_name = ident_tkn.get_name();
                self.assign(st_name, rhs);
            }
            AstNode::FnCallExpr {
                fn_ident_tkn,
                fn_params,
                is_std,
            } => self.fncallexpr(fn_ident_tkn, fn_params, *is_std),
            AstNode::PrimaryExpr { tkn } => {
                match &tkn.ty {
                    TokenTy::Val(v) => {
                        let op = Op::LoadC {
                            dest: self.nextreg(),
                            val: SifVal::Num(*v),
                        };
                        self.push_op(op);
                    }
                    TokenTy::Str(s) => {
                        let op = Op::LoadC {
                            dest: self.nextreg(),
                            val: SifVal::Str(s.clone()),
                        };
                        self.push_op(op);
                    }
                    TokenTy::Ident(i) => {
                        let op = Op::LoadN {
                            dest: self.nextreg(),
                            name: i.clone(),
                        };
                        self.push_op(op);
                    }
                    _ => {}
                };
            }
            _ => (),
        }
    }

    fn binop(&mut self, kind: BinOpKind, lhs: &AstNode, rhs: &AstNode) {
        let r0 = self.binarg(lhs);
        let r1 = self.binarg(rhs);

        let op = Op::Binary {
            kind: kind,
            src1: r0,
            src2: r1,
            dest: self.nextreg(),
        };
        self.push_op(op);
    }

    fn unop(&mut self, kind: UnOpKind, rhs: &AstNode) {
        let r0 = self.binarg(rhs);
        let op = Op::Unary {
            kind: kind,
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
                    let op = Op::LoadC { dest: d, val: sifv };
                    self.push_op(op);
                }
                TokenTy::True => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(true);
                    let op = Op::LoadC { dest: d, val: sifv };
                    self.push_op(op);
                }
                TokenTy::False => {
                    let d = self.nextreg();
                    let sifv = SifVal::Bl(false);
                    let op = Op::LoadC { dest: d, val: sifv };
                    self.push_op(op);
                }
                TokenTy::Ident(i) => {
                    let d = self.nextreg();
                    let op = Op::LoadN {
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

    fn ret(&mut self, ret_expr: &Option<Box<AstNode>>) {
        match ret_expr {
            Some(exp) => {
                self.expr(exp);
                // return moves the value from last reg into the frr register
                let op = Op::FnRetR {
                    src: self.prevreg(),
                };
                self.push_op(op);
            }
            None => self.push_op(Op::FnRet),
        };
    }

    fn fndecl(&mut self, ident_tkn: &Token, fn_params: &AstNode, fn_body: &AstNode) {
        let fn_name = ident_tkn.get_name();
        let mut param_names = Vec::new();
        let mut stkops = Vec::new();

        match fn_params {
            AstNode::FnParams { params } => {
                for p in params {
                    match p {
                        AstNode::PrimaryExpr { tkn } => {
                            // TODO: probably don't need to pop and then store params
                            // here for subsequent loads. We should just refer to the proper
                            // reg instead (this could be done in an optimizer)
                            param_names.push(tkn.get_name());
                            let stkop = Op::FnStackPop {
                                dest: self.nextreg(),
                            };
                            stkops.push(stkop);
                            let strop = Op::StoreR {
                                src: self.prevreg(),
                                name: tkn.get_name(),
                            };
                            stkops.push(strop);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        };

        // set our section to the decl section for function declaration instructions.
        self.decl_scope = true;

        self.push_op(Op::Fn {
            name: fn_name,
            params: param_names,
        });

        // Add the stack pop operations for param handling.
        for op in stkops {
            self.push_op(op);
        }

        self.newlbl();
        // TODO: param scoping - need to be able to access names in vm using the stack
        self.block(fn_body);
        self.newlbl();

        // go back to code section.
        self.decl_scope = false;
    }

    fn vardecl(&mut self, tkn: &Token, rhs: Option<Box<AstNode>>) {
        let st_name = tkn.get_name();
        if rhs.is_none() {
            // We generate a store for an empty value here, to ensure that the name is present
            // in memory if we try to assign to it later. We can detect null value accesses at some
            // point if we want to, or we can leave it to runtime.
            let op = Op::StoreC {
                val: SifVal::Null,
                name: st_name,
            };
            self.push_op(op);
            return;
        }

        self.assign(st_name, &rhs.unwrap());
    }

    pub fn assign(&mut self, st_name: String, rhs: &AstNode) {
        match rhs {
            AstNode::PrimaryExpr { tkn } => self.match_primary_assign(&st_name, &tkn),
            AstNode::Table {
                ident_tkn: _,
                items,
            } => self.match_table_assign(&st_name, items),
            AstNode::Array {
                ident_tkn, body, ..
            } => self.arraydecl(ident_tkn, body),
            AstNode::ArrayAccess { array_tkn, index } => self.arrayaccess(array_tkn, index),
            AstNode::FnCallExpr {
                fn_ident_tkn,
                fn_params,
                is_std,
            } => self.fn_call_assign(&st_name, &fn_ident_tkn, fn_params, *is_std),
            _ => {
                // We assume that if we aren't assigning a declaration to a constant, we are using an
                // expression. We store based on the correct register from the expression.
                self.expr(&rhs);
                let op = Op::StoreR {
                    name: st_name,
                    src: self.prevreg(),
                };
                self.push_op(op);
            }
        };
    }

    fn match_primary_assign(&mut self, st_name: &String, tkn: &Token) {
        match &tkn.ty {
            TokenTy::Val(v) => {
                self.push_op(Op::StoreC {
                    val: SifVal::Num(*v),
                    name: st_name.clone(),
                });
            }
            TokenTy::Str(s) => {
                self.push_op(Op::StoreC {
                    val: SifVal::Str(s.clone()),
                    name: st_name.clone(),
                });
            }
            TokenTy::Ident(i) => {
                self.push_op(Op::StoreN {
                    srcname: i.clone(),
                    destname: st_name.clone(),
                });
            }
            _ => {}
        };
    }

    fn match_table_assign(&mut self, st_name: &String, items: &AstNode) {
        self.push_op(Op::StoreC {
            val: SifVal::Tab(HashMap::new()),
            name: st_name.clone(),
        });

        match items {
            AstNode::ItemList { items } => {
                for (k, v) in items.iter() {
                    self.expr(v);
                    let tabop = Op::TblI {
                        tabname: st_name.clone(),
                        key: k.clone(),
                        src: self.prevreg(),
                    };
                    self.push_op(tabop);
                }
            }
            _ => {}
        };
    }

    fn fn_call_assign(
        &mut self,
        st_name: &String,
        fn_ident_tkn: &Token,
        fn_params: &Vec<AstNode>,
        is_std: bool,
    ) {
        self.fncallexpr(fn_ident_tkn, fn_params, is_std);

        // After the call returns, move the frr register to the next
        // available reg, and then store that register in the variable
        // being assigned to.
        let strop = Op::StoreR {
            src: self.prevreg(),
            name: st_name.clone(),
        };
        self.push_op(strop);
    }

    fn fncallexpr(&mut self, fn_ident_tkn: &Token, fn_params: &Vec<AstNode>, is_std: bool) {
        for param in fn_params {
            self.expr(param);
            let param_op = Op::FnStackPush {
                src: self.prevreg(),
            };
            self.push_op(param_op)
        }

        match is_std {
            true => {
                self.push_op(Op::StdCall {
                    name: fn_ident_tkn.get_name(),
                    param_count: fn_params.len(),
                });
            }
            false => {
                self.push_op(Op::Call {
                    name: fn_ident_tkn.get_name(),
                    param_count: fn_params.len(),
                });
            }
        };

        // After the call returns, move the frr register to the next
        // available reg, and then store that register in the variable
        // being assigned to.
        let frrop = Op::MvFRR {
            dest: self.nextreg(),
        };
        self.push_op(frrop);
    }
}
