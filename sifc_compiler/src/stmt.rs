use crate::{
    compiler::Compiler,
    opc::{BinOpKind, JmpOpKind, Op},
    sifv::SifVal,
};

use sifc_parse::ast::AstNode;

/// Contains compiler functions for if-stmts and for-stmts.

impl<'c> Compiler<'c> {
    pub fn ifstmt(
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
        let final_jmp_idx = (self.lblcnt() + 2) + (elif_exprs.len() * 2) + else_stmts.len();
        let mut el_jmp_idx = final_jmp_idx;
        if else_stmts.len() > 0 {
            el_jmp_idx = el_jmp_idx - 1;
        }

        // This initial conditional jump instruction appears after the conditional has
        // been evaluated above. If the conditional is false, we jump to the else block, or,
        // if the else block doesn't exist, to the end of the if statement.
        let jmp_op = Op::JumpCnd {
            kind: JmpOpKind::Jmpf,
            src: self.prevreg(),
            lbl: self.buildlbl(el_jmp_idx),
            lblidx: el_jmp_idx,
        };
        self.push_op(jmp_op);
        self.newlbl();

        // Generate statements for when the condition expression is true. Afterwards,
        // we jump always to the end of the if statement, so we do not run the instructions
        // contained in the else block.
        self.block(if_stmts);
        let jmpa_op = Op::JumpA {
            lbl: self.buildlbl(final_jmp_idx),
            lblidx: final_jmp_idx,
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
        self.newlbl();
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
                    kind: JmpOpKind::Jmpf,
                    src: self.prevreg(),
                    lbl: self.buildlbl(next_elif_jmp_idx),
                    lblidx: next_elif_jmp_idx,
                };
                self.push_op(jmp_op);
                self.newlbl();

                // After we generate the statements for the elif block, we jump out of the
                // if statement, skipping the else block if it exists.
                self.block(stmts);
                let jmpa_op = Op::JumpA {
                    lbl: self.buildlbl(final_jmp_idx),
                    lblidx: final_jmp_idx,
                };
                self.push_op(jmpa_op);
            }
            _ => {}
        };
    }

    pub fn forstmt(&mut self, var_list: &AstNode, in_expr_list: &AstNode, stmts: &AstNode) {
        self.newlbl();

        // Load the index register and set it to 0 initially.
        let idx_reg = self.nextreg();
        let (idx_name, local_name) = self.names_from_identpair(var_list);

        let loop_var_name = match in_expr_list {
            AstNode::PrimaryExpr { tkn, .. } => tkn.get_name(),
            _ => panic!("invalid expression list in ast!"),
        };

        // TODO: right now we assume that the value we are looping over is always an array.
        // later we could determine this by examining the in_expr_list node for other kinds.
        self.push_op(Op::StoreC {
            name: idx_name.clone(),
            val: SifVal::Num(0.0),
        });

        let size_reg = self.nextreg();

        // Load array size into size register.
        self.push_op(Op::LoadArrs {
            name: loop_var_name.clone(),
            dest: size_reg,
        });

        // This new label is used to jump to at the end of the loop. We can perform the previous
        // steps outside of the actual loop body and at least save a few instructions inside
        // the loop. We store this in loop_lbl in case we need to refer to it later on, and
        // to make sure we always jump to this label at the end of the loop.
        self.newlbl();
        let loop_lbl = self.lblcnt();

        // 1. Load the index name into the index reg at the start of each loop iteration.
        // 2. Load the array value into the local register
        // 3. Store the array value into the local name at each iteration, so if it
        // is accessed by name we return the correct contents.
        self.push_op(Op::LoadN {
            dest: idx_reg,
            name: idx_name.clone(),
        });

        let local_reg = self.nextreg();
        self.push_op(Op::LoadArrv {
            name: loop_var_name.clone(),
            idx_reg: idx_reg,
            dest: local_reg,
        });
        self.push_op(Op::StoreR {
            name: local_name.clone(),
            src: local_reg,
        });

        // Generate instructions for the actual loop statements.
        self.block(stmts);

        // Increment index register and store it again.
        self.push_op(Op::Incrr { src: idx_reg });
        self.push_op(Op::StoreR {
            name: idx_name.clone(),
            src: idx_reg,
        });

        // Compare the index register to the size register. If index is >=
        // our defined loop size, we fall through to the next instructions. If not,
        // we jump back to the loop start label.
        let idx_cmp = Op::Binary {
            kind: BinOpKind::Lt,
            src1: idx_reg,
            src2: size_reg,
            dest: self.nextreg(),
        };
        self.push_op(idx_cmp);

        let idx_jmp = Op::JumpCnd {
            kind: JmpOpKind::Jmpt,
            src: self.prevreg(),
            lbl: self.currlbl(),
            lblidx: loop_lbl,
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
}
