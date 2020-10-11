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

    fn blocks(&mut self, blocks: Vec<AstNode>) {
        for block in blocks {
            self.block(&block);
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

    fn arraydecl(&mut self, ident_tkn: &Token, body: &Option<Box<AstNode>>) {
        // Array declarations use a vec type wrapped in SifVal. This allows arrays
        // to contain multiple types of values, but also causes overhead in
        // memory allocation since we do not size the array and allocate in the heap
        // here. This is far easier to implement, but should be less efficient.
        let name = ident_tkn.get_name();

        match body {
            Some(ast) => {
                let items = self.arrayitems(ast);
                self.push_op(Op::StoreC {
                    ty: OpTy::Stc,
                    name: name,
                    val: SifVal::Arr(items),
                });
            }
            None => {
                self.push_op(Op::StoreC {
                    ty: OpTy::Stc,
                    name: name,
                    val: SifVal::Arr(Vec::new()),
                });
            }
        };
    }

    fn arrayitems(&mut self, ast: &AstNode) -> Vec<SifVal> {
        let mut vals = Vec::new();

        match ast {
            AstNode::ArrayItems { items } => {
                for item in items {
                    match item {
                        AstNode::PrimaryExpr { tkn } => {
                            let sv = self.val_from_primary(tkn);
                            vals.push(sv);
                        }
                        _ => {} // TODO: need to process exprs inside array decls
                    }
                }
            }
            _ => {}
        };
        vals
    }

    fn val_from_primary(&self, tkn: &Token) -> SifVal {
        // TODO: what about idents inside array decls?
        match &tkn.ty {
            TokenTy::Val(v) => SifVal::Num(*v),
            TokenTy::Str(s) => SifVal::Str(s.clone()),
            TokenTy::True => SifVal::Bl(true),
            TokenTy::False => SifVal::Bl(false),
            _ => SifVal::Null,
        }
    }

    fn ifstmt(
        &mut self,
        cond_expr: &AstNode,
        if_stmts: &AstNode,
        elif_exprs: Vec<AstNode>,
        else_stmts: Vec<AstNode>,
    ) {
        // Generate condition expression
        self.expr(cond_expr);

        // The jmp labels for if statements are calculated as follows:
        // 1. The initial if condition and statements take two labels
        // 2. Each elif takes two labels, one for the condition and one for the
        //    statements. This means we jump past them all from the initial
        //    if statements: (# elifs * 2)
        // 3. An else block takes 1 label, because there is no condition.
        //
        // We define two indices for this purpose:
        // First, final_jmp_idx represents the label after the entire if block
        // is completed.
        // Second, el_jmp_idx represents the label of the optional else statement, which
        // we must jump to from failed condition expressions if it exists.
        let final_jmp_idx = (self.lbl_cnt + 2) + (elif_exprs.len() * 2) + else_stmts.len();
        let mut el_jmp_idx = final_jmp_idx;
        if else_stmts.len() > 0 {
            el_jmp_idx = el_jmp_idx - 1;
        }

        // This initial conditional jump instruction appears after the conditional has
        // been evaluated above. If the conditional is false, we jump to the else block, or,
        // if the else block doesn't exist, to the end of the if statement.
        let jmp_op = Op::JumpCnd {
            ty: OpTy::Jmpf,
            src: self.prevreg(),
            lbl: self.buildlbl(el_jmp_idx),
        };
        self.push_op(jmp_op);
        self.newlbl();

        // Generate statements for when the condition expression is true. Afterwards,
        // we jump always to the end of the if statement, so we do not run the instructions
        // contained in the else block.
        self.block(if_stmts);
        let jmpa_op = Op::JumpA {
            ty: OpTy::Jmp,
            lbl: self.buildlbl(final_jmp_idx),
        };
        self.push_op(jmpa_op);

        // Generate statements for elif nodes, if any. More label calculations are done here:
        // each false elif condition should jump to the next possible elif, if it exists. If
        // it does not, it should jump to the else block. If the else block doesn't exist, we
        // jump to the end of the if statement.
        // The label initially points to the else block index. However, if we have additional
        // elif blocks to generate for, we alter the label so we jump to those conditionals
        // by reducing the label count by 2 to accomodate for the 2 labels needed by the elif.
        for (i, ee) in elif_exprs.iter().enumerate() {
            self.newlbl();

            let mut jmp_lbl = el_jmp_idx;
            if i != elif_exprs.len() - 1 {
                jmp_lbl = jmp_lbl - 2;
            }

            // We pass in jmp_lbl for the next elif expr, and final_jmp_idx to
            // get to the end of the if statement.
            self.elif(ee, jmp_lbl, final_jmp_idx);
        }

        // Generate statements for else nodes. No additional labeling is needed here,
        // as the else will fall through to subsequent instructions after being evaluated.
        if else_stmts.len() != 0 {
            self.newlbl();
            self.blocks(else_stmts);
        }
    }

    /// Generates instructions for an elif node. This takes in two jump label indices:
    /// next_elif_jmp_idx: the index for the next subsequent elif block, if any.
    /// final_jmp_idx: the label index for the end of the if statement.
    fn elif(&mut self, elif: &AstNode, next_elif_jmp_idx: usize, final_jmp_idx: usize) {
        match elif {
            AstNode::ElifStmt { cond_expr, stmts } => {
                self.expr(cond_expr);

                // If the condition is false, we go to the next elif condition expression.
                // If the next elif condition doesn't exist, then this will jump to the
                // end of the if statement (the index should be the same as final_jmp_idx).
                let jmp_op = Op::JumpCnd {
                    ty: OpTy::Jmpf,
                    src: self.prevreg(),
                    lbl: self.buildlbl(next_elif_jmp_idx),
                };
                self.push_op(jmp_op);
                self.newlbl();

                // After we generate the statements for the elif block, we jump out of the
                // if statement, skipping the else block if it exists.
                self.block(stmts);
                let jmpa_op = Op::JumpA {
                    ty: OpTy::Jmp,
                    lbl: self.buildlbl(final_jmp_idx),
                };
                self.push_op(jmpa_op);
            }
            _ => {}
        };
    }

    // TODO: need array decls
    fn forstmt(&mut self, var_list: &AstNode, in_expr_list: &AstNode, stmts: &AstNode) {
        self.newlbl();

        // Load the index register and set it to 0 initially.
        let idx_reg = self.nextreg();
        let (idx_name, local_name) = self.names_from_identpair(var_list);

        // TODO: right now we assume that the value we are looping over is always an array
        self.push_op(Op::StoreC {
            ty: OpTy::Stc,
            name: idx_name.clone(),
            val: SifVal::Num(0.0),
        });

        self.newlbl();

        // Load local var, and loop expr var into registers
        let local_reg = self.nextreg();
        let size_reg = self.nextreg();

        // TODO: right now we assume that the value we are looping over is always an array
        // Load the array index into register. We need to store the array size in the
        // size reg and handle the generation of array declarations.
        self.push_op(Op::LoadN {
            ty: OpTy::Ldn,
            dest: Rc::clone(&idx_reg),
            name: idx_name.clone(),
        });

        // generate stmts, then check if we need to jmp back to loop start
        self.block(stmts);

        // Increment index register and store it again
        self.push_op(Op::Incrr {
            ty: OpTy::Incrr,
            src: Rc::clone(&idx_reg),
        });
        self.push_op(Op::StoreR {
            ty: OpTy::Str,
            name: idx_name.clone(),
            src: Rc::clone(&idx_reg),
        });

        // Compare the index register to the size register. If index is >=
        // our defined loop size, we fall through to the next instructions. If not,
        // we jump back to the loop start label.
        let idx_cmp = Op::Binary {
            ty: OpTy::Lt,
            src1: Rc::clone(&idx_reg),
            src2: Rc::clone(&size_reg),
            dest: self.nextreg(),
        };
        self.push_op(idx_cmp);

        let idx_jmp = Op::JumpCnd {
            ty: OpTy::Jmpt,
            src: self.prevreg(),
            lbl: self.currlbl(),
        };
        self.push_op(idx_jmp);
    }

    /// Processes AstNode::IdentPair and returns a tuple of the names inside the node
    fn names_from_identpair(&mut self, var_list: &AstNode) -> (String, String) {
        let mut n1 = String::new();
        let mut n2 = String::new();

        match var_list {
            AstNode::IdentPair { idents } => {
                // the vector of idents should contain no more than 2 items
                match &idents[0] {
                    AstNode::PrimaryExpr { tkn } => n1 = tkn.get_name(),
                    _ => {}
                };
                match &idents[1] {
                    AstNode::PrimaryExpr { tkn } => n2 = tkn.get_name(),
                    _ => {}
                };
            }
            _ => panic!("invalid ident pair ast found!"),
        };

        (n1, n2)
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

    fn upd_op_at_idx(&mut self, op: Op, idx: usize) {
        let i = &mut self.ops[idx];
        i.op = op;
    }

    fn currlbl(&self) -> String {
        format!("lbl{}", self.lbl_cnt)
    }

    fn newlbl(&mut self) {
        self.lbl_cnt = self.lbl_cnt + 1;
    }

    fn buildlbl(&self, cnt: usize) -> String {
        format!("lbl{}", cnt)
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

    fn build_name(&mut self, name: String) -> String {
        format!("ident:{}", name)
    }
}
