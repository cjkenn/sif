use crate::{compiler::Compiler, opc::Op, sifv::SifVal};

use sifc_parse::{
    ast::AstNode,
    token::{Token, TokenTy},
};

impl<'c> Compiler<'c> {
    /// Compiles and generates IR for AstNode::ArrayDecl types.
    pub fn arraydecl(&mut self, ident_tkn: &Token, body: &Option<Box<AstNode>>) {
        // Array declarations use a vec type wrapped in SifVal. This allows arrays
        // to contain multiple types of values, but also causes overhead in
        // memory allocation since we do not size the array and allocate in the heap
        // here. This is far easier to implement, but should be less efficient.
        let name = ident_tkn.get_name();

        match body {
            Some(ast) => {
                let items = self.arrayitems(ast);
                self.push_op(Op::Stc {
                    name: name,
                    val: SifVal::Arr(items),
                });
            }
            None => {
                self.push_op(Op::Stc {
                    name: name,
                    val: SifVal::Arr(Vec::new()),
                });
            }
        };
    }

    /// Compiles instructions for AstNode::ArrayAccess types.
    pub fn arrayaccess(&mut self, array_tkn: &Token, index_expr: &AstNode) {
        let name = array_tkn.get_name();
        self.expr(index_expr);

        let op = Op::Ldav {
            name: name,
            idx_reg: self.prevreg(),
            dest: self.nextreg(),
        };

        self.push_op(op);
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
                        // TODO: need to process exprs inside array decls
                        _ => unimplemented!("cannot generate ir for exprs in array decls"),
                    }
                }
            }
            _ => {}
        };
        vals
    }

    fn val_from_primary(&self, tkn: &Token) -> SifVal {
        // TODO: idents inside array decls. Should use a new op for this as the arrays are
        // not considered constant values now
        match &tkn.ty {
            TokenTy::Val(v) => SifVal::Num(*v),
            TokenTy::Str(s) => SifVal::Str(s.clone()),
            TokenTy::True => SifVal::Bl(true),
            TokenTy::False => SifVal::Bl(false),
            TokenTy::Ident(_i) => unimplemented!("cannot generate ir for idents in array decls"),
            _ => SifVal::Null,
        }
    }
}
