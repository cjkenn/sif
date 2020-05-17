use crate::{
    ast::AstNode,
    error::{ParseErr, ParseErrTy},
    lex::Lexer,
    symbol_table::SymTab,
    token::{Token, TokenTy},
};

use std::{collections::HashMap, rc::Rc};

const FN_PARAM_MAX_LEN: usize = 64;

/// ParserResult handles the result from parsing a file. This contains an optional
/// AST structure, as well as a flag indicating whether or not continuable errors
/// were encountered during the parsing phase. This is returned from the parse()
/// method, and should be checked for errors before continuing further phases
/// of the compiler.
#[derive(Default)]
pub struct ParserResult {
    /// The resulting AST from parsing.
    pub ast: Option<AstNode>,

    /// Flag indicating if errors have ocurred during parsing.
    pub has_err: bool,

    /// Vec of errors that have been parsed.
    pub errors: Vec<ParseErr>,
}

pub struct Parser<'l, 's> {
    /// Reference to the lexer needed to get characters from the file.
    lexer: &'l mut Lexer,

    /// Reference to a symbol table, used to store symbols defined in this file.
    sym_tab: &'s mut SymTab,

    /// The current token from the lexer.
    curr_tkn: Token,

    /// AST node size.
    node_count: usize,

    /// Number of symbols encountered.
    sym_count: usize,

    /// Vec of errors parsed so far
    errors: Vec<ParseErr>,
}

impl<'l, 's> Parser<'l, 's> {
    pub fn new(lex: &'l mut Lexer, symt: &'s mut SymTab) -> Parser<'l, 's> {
        let firsttkn = lex.lex();

        Parser {
            lexer: lex,
            sym_tab: symt,
            curr_tkn: firsttkn,
            node_count: 1, // start at 1 because the entry node always has id 0
            sym_count: 0,
            errors: Vec::new(),
        }
    }

    /// Main entry point to the recursive descent parser. Calling this method will parse the entire
    /// file and return a result containing the AST and any parsing errors encountered.
    /// The error vector should be checked after parsing, and any errors should
    /// be handled before continuing to future compiler passes.
    pub fn parse(&mut self) -> ParserResult {
        let mut blocks: Vec<AstNode> = Vec::new();
        let mut found_err = false;

        while self.curr_tkn.ty != TokenTy::Eof {
            match self.decl() {
                Ok(a) => blocks.push(a),
                Err(e) => {
                    found_err = true;
                    e.emit();
                    match e.continuable() {
                        true => (),
                        false => break,
                    };
                }
            }
        }

        let head = AstNode::Program { blocks: blocks };

        ParserResult {
            ast: Some(head),
            has_err: found_err,
            errors: self.errors.clone(),
        }
    }

    fn decl(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::Let => self.var_decl(),
            TokenTy::Fn => self.fn_decl(),
            TokenTy::Record => self.record_decl(),
            TokenTy::Table => self.table_decl(),
            TokenTy::Array => self.array_decl(),
            _ => self.stmt(),
        }
    }

    fn stmt(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::If => self.if_stmt(),
            TokenTy::For => self.for_stmt(),
            TokenTy::Return => self.ret_stmt(),
            TokenTy::LeftBrace => self.block(),
            _ => self.expr_stmt(),
        }
    }

    fn block(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::LeftBrace)?;

        let mut decls = Vec::new();
        self.sym_tab.init_scope();

        loop {
            match self.curr_tkn.ty {
                TokenTy::RightBrace | TokenTy::Eof => break,
                _ => {
                    let result = self.decl()?;
                    decls.push(result);
                }
            };
        }

        self.expect(TokenTy::RightBrace)?;

        Ok(AstNode::Block {
            decls: decls,
            scope: self.sym_tab.level(),
        })
    }

    fn var_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Let)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        match self.curr_tkn.ty {
            TokenTy::Eq => {
                let lhs = self.expr()?;
                self.expect(TokenTy::Semicolon)?;

                let node = AstNode::VarDecl {
                    ident_tkn: ident_tkn.clone(),
                    is_global: self.sym_tab.is_global(),
                    lhs: Some(Box::new(lhs)),
                };

                self.sym_tab.store(&ident_tkn.get_name(), node.clone());
                Ok(node)
            }
            TokenTy::Semicolon => {
                self.expect(TokenTy::Semicolon)?;

                let node = AstNode::VarDecl {
                    ident_tkn: ident_tkn.clone(),
                    is_global: self.sym_tab.is_global(),
                    lhs: None,
                };

                self.sym_tab.store(&ident_tkn.get_name(), node.clone());
                Ok(node)
            }
            _ => Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                "expected either '=' or ';",
            )))),
        }
    }

    fn fn_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Fn)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        self.expect(TokenTy::LeftParen)?;
        let params = self.param_list()?;
        self.expect(TokenTy::RightParen)?;
        let body = self.block()?;

        let node = AstNode::FnDecl {
            ident_tkn: ident_tkn.clone(),
            fn_params: Box::new(params),
            fn_body: Box::new(body),
            scope: self.sym_tab.level(),
        };

        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    fn param_list(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::RightParen => {
                // this indicates an empty param list
                Ok(AstNode::FnParams { params: Vec::new() })
            }
            _ => {
                // we expect the param list to be all identifiers: fn f(x, y, z). We could
                // call expr() for each param, but we can short circuit the recursive calls
                // by just creating PrimaryExpr nodes here and adding them to the param list.
                let mut param_list = Vec::new();

                while self.curr_tkn.ty != TokenTy::RightParen {
                    if self.curr_tkn.ty == TokenTy::Eof {
                        return Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                            "unexpected end of file",
                        ))));
                    }

                    let maybe_ident_tkn = self.match_ident();
                    if maybe_ident_tkn.is_none() {
                        return Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                            "expected identifier",
                        ))));
                    }
                    let ident_tkn = maybe_ident_tkn.unwrap();

                    param_list.push(AstNode::PrimaryExpr { tkn: ident_tkn });

                    if self.curr_tkn.ty != TokenTy::RightParen {
                        self.expect(TokenTy::Comma)?;
                    }
                }

                Ok(AstNode::FnParams { params: param_list })
            }
        }
    }

    fn record_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Record)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        let body = self.block()?;
        let node = AstNode::RecordDecl {
            ident_tkn: ident_tkn.clone(),
            rec_body: Box::new(body),
        };
        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    fn table_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Table)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        let body = self.block()?;
        let node = AstNode::TableDecl {
            ident_tkn: ident_tkn.clone(),
            tab_body: Box::new(body),
        };
        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    fn array_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Array)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        self.expect(TokenTy::LeftBracket)?;

        let body = self.expr()?;

        self.expect(TokenTy::RightBracket)?;
        self.expect(TokenTy::Semicolon)?;

        let node = AstNode::ArrayDecl {
            ident_tkn: ident_tkn.clone(),
            arr_body: Box::new(body),
        };
        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    fn if_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::If)?;

        let if_cond = self.expr()?;
        let if_blck = self.block()?;

        let mut else_blck = Vec::new();
        let mut else_ifs = Vec::new();
        let mut else_cnt = 0;

        loop {
            match self.curr_tkn.ty {
                TokenTy::Elif => {
                    self.expect(TokenTy::Elif)?;

                    let elif_ast = self.expr()?;
                    let elif_blck = self.block()?;
                    let stmt_ast = AstNode::ElifStmt {
                        cond_expr: Box::new(elif_ast),
                        stmts: Box::new(elif_blck),
                    };
                    else_ifs.push(stmt_ast);
                }
                TokenTy::Else => {
                    else_cnt = else_cnt + 1;
                    self.expect(TokenTy::Else)?;
                    let blck = self.block()?;
                    else_blck.push(blck);
                }
                _ => break,
            };
        }

        if else_cnt > 1 {
            self.add_error(ParseErrTy::InvalidIfStmt);
        }

        Ok(AstNode::IfStmt {
            cond_expr: Box::new(if_cond),
            if_stmts: Box::new(if_blck),
            elif_exprs: else_ifs,
            else_stmts: else_blck,
        })
    }

    fn for_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::For)?;

        let var_list = self.ident_pair()?;

        self.expect(TokenTy::In)?;

        let in_expr_list = self.expr()?;
        let stmts = self.block()?;

        Ok(AstNode::ForStmt {
            var_list: Box::new(var_list),
            in_expr_list: Box::new(in_expr_list),
            stmts: Box::new(stmts),
        })
    }

    fn ident_pair(&mut self) -> Result<AstNode, ParseErr> {
        let mut idents = Vec::new();

        let maybe_first_ident = self.match_ident();
        if maybe_first_ident.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let first_ident = maybe_first_ident.unwrap();
        idents.push(AstNode::PrimaryExpr { tkn: first_ident });

        self.expect(TokenTy::Comma)?;

        let maybe_sec_ident = self.match_ident();
        if maybe_sec_ident.is_none() {
            return Err(self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier"))));
        }
        let second_ident = maybe_sec_ident.unwrap();
        idents.push(AstNode::PrimaryExpr { tkn: second_ident });

        Ok(AstNode::IdentPair { idents: idents })
    }

    fn ret_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Return)?;

        match self.curr_tkn.ty {
            TokenTy::Semicolon => {
                self.expect(TokenTy::Semicolon)?;

                Ok(AstNode::ReturnStmt { ret_expr: None })
            }
            _ => {
                let ret_expr = self.expr()?;
                self.expect(TokenTy::Semicolon)?;

                Ok(AstNode::ReturnStmt {
                    ret_expr: Some(Box::new(ret_expr)),
                })
            }
        }
    }

    fn expr_stmt(&mut self) -> Result<AstNode, ParseErr> {
        let node = self.expr()?;
        self.expect(TokenTy::Semicolon)?;
        Ok(AstNode::ExprStmt {
            expr: Box::new(node),
        })
    }

    fn expr(&mut self) -> Result<AstNode, ParseErr> {
        self.assign_expr()
    }

    fn assign_expr(&mut self) -> Result<AstNode, ParseErr> {
        let ast = self.or_expr()?;

        match self.curr_tkn.ty {
            TokenTy::Eq => {
                let op = self.curr_tkn.clone();
                self.expect(TokenTy::Eq)?;
                let rhs = self.assign_expr()?;

                match ast.clone() {
                    AstNode::PrimaryExpr { tkn } => {
                        match tkn.ty {
                            TokenTy::Ident(name) => {
                                let maybe_sym = self.sym_tab.retrieve(&name);
                                if maybe_sym.is_none() {
                                    return Err(self.add_error(ParseErrTy::UndeclSym(name)));
                                }

                                let var_node = maybe_sym.unwrap();
                                match var_node {
                                    AstNode::VarDecl {
                                        ident_tkn,
                                        is_global,
                                        ..
                                    } => {
                                        return Ok(AstNode::VarAssignExpr {
                                            ident_tkn: ident_tkn.clone(),
                                            is_global: is_global,
                                            rhs: Box::new(rhs),
                                        });
                                    }
                                    _ => {
                                        return Err(self.add_error(ParseErrTy::UndeclSym(name)));
                                    }
                                }
                            }
                            _ => {
                                return Err(self.add_error(ParseErrTy::InvalidAssign(
                                    tkn.ty.clone().to_string(),
                                )));
                            }
                        };
                    }
                    _ => {
                        return Err(self.add_error_w_pos(
                            op.line,
                            op.pos,
                            ParseErrTy::InvalidAssign(op.ty.to_string()),
                        ));
                    }
                }
            }
            _ => (),
        };

        Ok(ast)
    }

    fn or_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.and_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::PipePipe => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.and_expr()?;

                    ast = AstNode::LogicalExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn and_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.equality_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::AmpAmp => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.equality_expr()?;

                    ast = AstNode::LogicalExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn equality_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.compr_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::BangEq => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.compr_expr()?;

                    ast = AstNode::BinaryExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn compr_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.add_or_sub_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::Lt | TokenTy::LtEq | TokenTy::Gt | TokenTy::GtEq => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.add_or_sub_expr()?;

                    ast = AstNode::BinaryExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn add_or_sub_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.mul_or_div_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::Plus | TokenTy::Minus => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.mul_or_div_expr()?;

                    ast = AstNode::BinaryExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn mul_or_div_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.mod_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::Star | TokenTy::Slash => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.mod_expr()?;

                    ast = AstNode::BinaryExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn mod_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.unary_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::Percent => {
                    let op = self.curr_tkn.clone();

                    self.consume();

                    let rhs = self.unary_expr()?;

                    ast = AstNode::BinaryExpr {
                        op_tkn: op,
                        lhs: Box::new(ast),
                        rhs: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }

        Ok(ast)
    }

    fn unary_expr(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::Bang | TokenTy::Minus => {
                let op = self.curr_tkn.clone();

                self.consume();

                let rhs = self.unary_expr()?;

                return Ok(AstNode::UnaryExpr {
                    op_tkn: op,
                    rhs: Box::new(rhs),
                });
            }
            // TODO: array, table, record access
            _ => self.fn_call_expr(),
        }
    }

    fn fn_call_expr(&mut self) -> Result<AstNode, ParseErr> {
        let ast = self.primary_expr()?;
        let mut params = Vec::new();
        let ident_tkn = match ast {
            AstNode::PrimaryExpr { ref tkn } => Some(tkn.clone()),
            _ => None,
        };

        // If this is a class ident, we expect a period and then either a property name
        // or a function call. If this is a regular function ident, we expect an
        // opening paren next.
        match self.curr_tkn.ty {
            TokenTy::LeftParen => {
                self.expect(TokenTy::LeftParen)?;
                let params_list = self.param_list()?;
                self.expect(TokenTy::RightParen)?;
                match params_list {
                    AstNode::FnParams {
                        params: inner_params,
                    } => {
                        params = inner_params;
                    }
                    _ => (),
                };

                return Ok(AstNode::FnCallExpr {
                    fn_ident_tkn: ident_tkn.unwrap(),
                    fn_params: params,
                });
            }
            _ => (),
        };

        Ok(ast)
    }

    fn primary_expr(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty.clone() {
            TokenTy::Str(_) | TokenTy::Val(_) | TokenTy::True | TokenTy::False => {
                let ast = Ok(AstNode::PrimaryExpr {
                    tkn: self.curr_tkn.clone(),
                });
                self.consume();
                ast
            }
            TokenTy::Ident(ref ident_name) => {
                let ident_tkn = self.curr_tkn.clone();

                let maybe_ast = self.sym_tab.retrieve(ident_name);
                if maybe_ast.is_none() {
                    let err = self.add_error(ParseErrTy::UndeclSym(ident_name.to_string()));
                    self.consume();
                    return Err(err);
                }

                let ast = Ok(AstNode::PrimaryExpr { tkn: ident_tkn });

                self.consume();
                ast
            }
            TokenTy::LeftParen => {
                return self.group_expr();
            }
            _ => {
                let ty_str = self.curr_tkn.ty.to_string();
                let err = self.add_error(ParseErrTy::InvalidTkn(ty_str));
                self.consume();
                Err(err)
            }
        }
    }

    fn group_expr(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::LeftParen)?;
        let ast = self.expr()?;
        self.expect(TokenTy::RightParen)?;
        Ok(ast)
    }

    /// Check that the current token is the same as the one we expect. If it is, consume the
    /// token and advance. If it isn't report an error.
    fn expect(&mut self, tknty: TokenTy) -> Result<(), ParseErr> {
        if self.curr_tkn.ty == tknty {
            self.consume();
            Ok(())
        } else {
            let ty_str = self.curr_tkn.ty.to_string();
            let err_ty = ParseErrTy::TknMismatch(tknty.to_string(), ty_str);
            Err(ParseErr::new(self.curr_tkn.line, self.curr_tkn.pos, err_ty))
        }
    }

    /// Expects an identifier token to be passed in. If it is, returns a token that matches
    /// the identifier that we've parsed. If it's not, we return None and add an error
    /// to the error vec.
    fn match_ident(&mut self) -> Option<Token> {
        match self.curr_tkn.ty {
            TokenTy::Ident(_) => {
                let tkn = Some(self.curr_tkn.clone());
                self.consume();
                tkn
            }
            _ => {
                let ty_str = self.curr_tkn.ty.to_string();
                self.add_error(ParseErrTy::InvalidIdent(ty_str));
                None
            }
        }
    }

    /// Advance to the next token, discarded the previously read token.
    fn consume(&mut self) {
        self.curr_tkn = self.lexer.lex();
    }

    /// Push a parsing error onto the error vector.
    fn add_error(&mut self, ty: ParseErrTy) -> ParseErr {
        let err = ParseErr::new(self.curr_tkn.line, self.curr_tkn.pos, ty);
        self.errors.push(err.clone());
        err
    }

    /// Report a parsing error at a given location with a provided error type.
    fn add_error_w_pos(&mut self, line: usize, pos: usize, ty: ParseErrTy) -> ParseErr {
        let err = ParseErr::new(line, pos, ty);
        self.errors.push(err.clone());
        err
    }
}
