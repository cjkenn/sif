use crate::token::Token;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum AstNode {
    Program {
        blocks: Vec<AstNode>,
    },

    Block {
        decls: Vec<AstNode>,
        scope: usize,
    },

    IfStmt {
        cond_expr: Box<AstNode>,
        if_stmts: Box<AstNode>,
        elif_exprs: Vec<AstNode>,
        else_stmts: Vec<AstNode>,
    },

    ElifStmt {
        cond_expr: Box<AstNode>,
        stmts: Box<AstNode>,
    },

    ForStmt {
        var_list: Box<AstNode>,
        in_expr_list: Box<AstNode>,
        stmts: Box<AstNode>,
    },

    ReturnStmt {
        ret_expr: Option<Box<AstNode>>,
    },

    ExprStmt {
        expr: Box<AstNode>,
    },

    VarDecl {
        ident_tkn: Token,
        is_global: bool,
        rhs: Option<Box<AstNode>>,
    },

    FnDecl {
        ident_tkn: Token,
        fn_params: Box<AstNode>,
        fn_body: Box<AstNode>,
        scope: usize,
    },

    FnParams {
        params: Vec<AstNode>,
    },

    IdentPair {
        idents: Vec<AstNode>,
    },

    ItemList {
        items: HashMap<String, AstNode>,
    },

    TableItem {
        key: Box<AstNode>,
        val: Box<AstNode>,
    },

    Table {
        ident_tkn: Token,
        items: Box<AstNode>,
    },

    TableAccess {
        table_tkn: Token,
        index: Box<AstNode>,
    },

    Array {
        ident_tkn: Token,
        body: Option<Box<AstNode>>,
        len: usize,
    },

    ArrayItems {
        items: Vec<AstNode>,
    },

    ArrayAccess {
        array_tkn: Token,
        index: Box<AstNode>,
    },

    FnCallExpr {
        fn_ident_tkn: Token,
        fn_params: Vec<AstNode>,
        is_std: bool,
    },

    VarAssignExpr {
        ident_tkn: Token,
        is_global: bool,
        rhs: Box<AstNode>,
    },

    BinaryExpr {
        op_tkn: Token,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },

    UnaryExpr {
        op_tkn: Token,
        rhs: Box<AstNode>,
    },

    PrimaryExpr {
        tkn: Token,
    },
}

impl AstNode {
    pub fn is_primary_expr(&self) -> bool {
        match self {
            AstNode::PrimaryExpr { .. } => true,
            _ => false,
        }
    }

    pub fn get_fn_params(&self) -> Vec<AstNode> {
        match self {
            AstNode::FnCallExpr {
                fn_ident_tkn: _,
                fn_params,
                ..
            } => fn_params.clone(),
            _ => Vec::new(),
        }
    }
}
