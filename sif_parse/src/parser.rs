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

        Ok(AstNode::FnDecl {
            ident_tkn: ident_tkn.clone(),
            fn_params: Box::new(params),
            fn_body: Box::new(body),
            scope: self.sym_tab.level(),
        })
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
        unimplemented!()
    }

    fn table_decl(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn array_decl(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn if_stmt(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn for_stmt(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn ret_stmt(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn expr_stmt(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn expr(&mut self) -> Result<AstNode, ParseErr> {
        self.assign_expr()
    }

    fn assign_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn or_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn and_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn equality_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn compr_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn binop_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn unary_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
    }

    fn primary_expr(&mut self) -> Result<AstNode, ParseErr> {
        unimplemented!()
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
}
