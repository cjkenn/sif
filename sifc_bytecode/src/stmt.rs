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

        // Insert a placeholder for the conditional jump. This well be updated later
        // when we have more label information and can compute the proper jump index.
        // We store the register and the idx for later, when the udpate is performed.
        let jmpcnd_idx = self.instr_count_in_scope();
        let jmpcnd_reg = self.prevreg();
        let jmpcnd_op_placeholder = Op::JmpCnd {
            kind: JmpOpKind::Jmpf,
            src: usize::MAX,
            lblidx: usize::MAX,
        };
        self.push_op(jmpcnd_op_placeholder);

        // Generate statements for when the condition expression is true.
        self.newlbl();
        self.block(if_stmts);

        // Like the conditional jump, we store a palceholder for the unconditional
        // jump instruction. We need to update this later when we know the exact
        // position to jump to.
        let jmpa_idx = self.instr_count_in_scope();
        let jmpa_op_placeholder = Op::Jmpa { lblidx: usize::MAX };
        self.push_op(jmpa_op_placeholder);

        let has_elifs = elif_exprs.len() != 0;
        let mut jmpcnd_lbl = self.lblcnt() + 1; // first elif label, if exists

        // Generate statements for elif nodes. Each elif generation returns the jump index
        // for the unconditional branch: these must be updated later to the correct index
        // once we know how many blocks we processed here. We store them in a vec and
        // overwrite them.
        let mut jmpa_op_idxs = Vec::new();
        for ee in elif_exprs {
            self.newlbl();

            let jmpa_idx = self.elif(&ee);
            jmpa_op_idxs.push(jmpa_idx);
        }

        // Generate statements for else nodes. No additional labeling is needed here,
        // as the else will fall through to subsequent instructions after being evaluated.
        let has_else = else_stmts.len() != 0;
        if has_else && !has_elifs {
            jmpcnd_lbl = self.lblcnt() + 1;
        }

        if has_else {
            self.newlbl();
            self.blocks(else_stmts);
        }

        self.newlbl();

        // Updating the first conditional jump as follows:
        // 1. No else or elifs: jump to end of block
        // 2. Else, no elif: jump to start of else
        // 3. Elif, no else: jump to first elif
        // 4. Elif, else: jump to first elif
        if !has_else && !has_elifs {
            jmpcnd_lbl = self.lblcnt();
        }
        let jmp_op_real = Op::JmpCnd {
            kind: JmpOpKind::Jmpf,
            src: jmpcnd_reg,
            lblidx: jmpcnd_lbl,
        };
        self.update_op_at(jmpcnd_idx, jmp_op_real);

        // Update jmpa indexes in any elifs to point to the very end of this block, as they should
        // always skip every other elif/else condition.
        for idx in jmpa_op_idxs {
            let jmpa_op_real = Op::Jmpa {
                lblidx: self.lblcnt(),
            };
            self.update_op_at(idx, jmpa_op_real);
        }

        // Update the always executed jump to go to the last label in this current block, as (like
        // the elif labels) we should skip everything from this jump.
        let jmpa_op_real = Op::Jmpa {
            lblidx: self.lblcnt(),
        };
        self.update_op_at(jmpa_idx, jmpa_op_real);

        // Push a Nop in: this fixes jumps to "empty" blocks (ie. make an exit block)
        self.push_op(Op::Nop);
    }

    /// Generates instructions for an elif node.
    /// Returns the jmpa program index that should be updated later to change the jump
    /// index to be the last out of the elif.
    fn elif(&mut self, elif: &AstNode) -> usize {
        match elif {
            AstNode::ElifStmt { cond_expr, stmts } => {
                self.expr(cond_expr);

                // Similar to regular if stmt generation, we store the index and register for
                // the conditional jump so we can update it later once more label information
                // is available.
                let jmpcnd_idx = self.instr_count_in_scope();
                let jmpcnd_reg = self.prevreg();

                let jmp_op_placeholder = Op::JmpCnd {
                    kind: JmpOpKind::Jmpf,
                    src: usize::MAX,
                    lblidx: usize::MAX,
                };
                self.push_op(jmp_op_placeholder);
                self.newlbl();

                // After we generate the statements for the elif block, we jump out of the
                // if statement, skipping the else block if it exists.
                self.block(stmts);

                // Update the conditional jump.
                let jmp_op_real = Op::JmpCnd {
                    kind: JmpOpKind::Jmpf,
                    src: jmpcnd_reg,
                    lblidx: self.lblcnt() + 1,
                };
                self.update_op_at(jmpcnd_idx, jmp_op_real);

                // Record the location of the jmpa label so it can be updated later
                // after all elifs are processed.
                let jmpa_place = self.instr_count_in_scope();
                let jmpa_op = Op::Jmpa { lblidx: usize::MAX };
                self.push_op(jmpa_op);

                jmpa_place
            }
            _ => usize::MAX,
        }
    }

    pub fn forstmt(&mut self, var_list: &AstNode, in_expr_list: &AstNode, stmts: &AstNode) {
        // Load the index register and set it to 0 initially.
        let idx_reg = self.nextreg();
        let (idx_name, local_name) = self.names_from_identpair(var_list);

        let loop_var_name = match in_expr_list {
            AstNode::PrimaryExpr { tkn, .. } => tkn.get_name(),
            _ => panic!("invalid expression list in ast!"),
        };

        // TODO: right now we assume that the value we are looping over is always an array.
        // later we could determine this by examining the in_expr_list node for other kinds.
        self.push_op(Op::Stc {
            name: idx_name.clone(),
            val: SifVal::Num(0.0),
        });

        let size_reg = self.nextreg();

        // Load array size into size register.
        self.push_op(Op::Ldas {
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
        self.push_op(Op::Ldn {
            dest: idx_reg,
            name: idx_name.clone(),
        });

        let local_reg = self.nextreg();
        self.push_op(Op::Ldav {
            name: loop_var_name.clone(),
            idx_reg: idx_reg,
            dest: local_reg,
        });
        self.push_op(Op::Str {
            name: local_name.clone(),
            src: local_reg,
        });

        // Generate instructions for the actual loop statements.
        self.block(stmts);

        // Increment index register and store it again.
        self.push_op(Op::Incrr { src: idx_reg });
        self.push_op(Op::Str {
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

        let idx_jmp = Op::JmpCnd {
            kind: JmpOpKind::Jmpt,
            src: self.prevreg(),
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
