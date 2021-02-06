use crate::{
    ast::AstNode,
    lex::Lexer,
    symtab::SymTab,
    token::{Token, TokenTy},
};

use sifc_err::{
    err::SifErr,
    parse_err::{ParseErr, ParseErrTy},
};

use std::collections::HashMap;
use std::io::Read;

/// Maximum amount of params allowed in function declarations.
const FN_PARAM_MAX_LEN: usize = 64;

/// [`ParserResult`] handles the result from parsing a file. This contains an optional
/// AST structure, as well as a flag indicating whether or not continuable errors
/// were encountered during the parsing phase. This is returned from the parse()
/// method, and should be checked for errors before continuing further phases
/// of the compiler.
#[derive(Default)]
pub struct ParserResult {
    /// The resulting AST from parsing. This will be None if parsing failed with
    /// non-continuable errors.
    pub ast: Option<AstNode>,

    /// Flag indicating if errors have ocurred during parsing.
    pub has_err: bool,

    /// Vec of errors that have been parsed. It is possible to encounter
    /// several continuable errors.
    pub errors: Vec<ParseErr>,
}

/// [`Parser`] implements a top-down, LL(1) recursive descent parser for the sif grammar.
/// The grammar is fairly simple, and the structure of the parser follows
/// pretty clearly from it. There are a couple of things to note:
///
/// 1. Operator precedence is encoded into the grammar itself. This means that parsing expressions
/// begins at the most general precedence (expr), and ends at the most specific (primary expressions
/// which are usually primitive values).
///
/// 2. The parser contains a flag to enabel/disable symbol table checking. This allows some control
/// over when to emit parsing errors for undefined or poorly defined symbols, in cases where
/// the parser may know that the symbols do not need to be defined when parsing certain constructs.
///
/// 3. Some errors encountered during parsing can be marked continuable, which indicates that
/// parsing can continue after the error is recorded.
///
/// In general, parsing methods will match on tokens using `expect()` or `optional()`, and recurse
/// to the correct production based on the current available token, `curr_tkn`. Each method makes use
/// of the `Result` trait, and should also make use of the `?` operator when making calls to other
/// production methods.
pub struct Parser<'l, 's, T>
where
    T: Read,
{
    /// Reference to the lexer needed to get characters from the file.
    lexer: &'l mut Lexer<T>,

    /// Reference to a symbol table, used to store symbols defined in this file.
    sym_tab: &'s mut SymTab,

    /// The current token from the lexer.
    curr_tkn: Token,

    /// Vec of errors parsed so far.
    errors: Vec<ParseErr>,

    /// Flag indicating whether or not potential identifiers should be looked up
    /// the symbol table before attempting to access or assign them. This is true
    /// almost all the time, but parsing table access is simplified for now
    /// by disabling checking the table definition.
    should_check_sym_tab: bool,
}

impl<'l, 's, T> Parser<'l, 's, T>
where
    T: Read,
{
    /// Creates a new parser. Makes an initial call to `lex()` in the supplied lexer,
    /// in order to fill the `curr_tkn` field.
    pub fn new(lex: &'l mut Lexer<T>, symt: &'s mut SymTab) -> Parser<'l, 's, T> {
        let firsttkn = lex.lex();

        Parser {
            lexer: lex,
            sym_tab: symt,
            curr_tkn: firsttkn,
            errors: Vec::new(),
            should_check_sym_tab: true,
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

    /// Parses a declaration rule.
    ///
    /// decl ::= vardecl | funcdecl | { stmt } ;
    fn decl(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::Var => self.var_decl(),
            TokenTy::Fn => self.fn_decl(),
            _ => self.stmt(),
        }
    }

    /// Parses a statement. There are currently 5 kinds of statements, including the
    /// Block statement, which is a brace delimited list of other declarations.
    ///
    /// stmt ::= ifstmt   |
    ///          forstmt  |
    ///          exprstmt |
    ///          retstmt  |
    ///          block    ;
    fn stmt(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::If => self.if_stmt(),
            TokenTy::For => self.for_stmt(),
            TokenTy::Return => self.ret_stmt(),
            TokenTy::LeftBrace => self.block(None),
            _ => self.expr_stmt(),
        }
    }

    /// Parses a block statement. Takes in an optional list of bindings that should
    /// be inserted into the symbol table while parsing the block. This makes it
    /// easier to bind variables to the scope defined in for loops and function declarations.
    ///
    /// block ::= "{" { decl } "}" ;
    fn block(&mut self, bindings: Option<Vec<AstNode>>) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::LeftBrace)?;

        let mut decls = Vec::new();

        // Make a new scope for the block, and insert optional block scope bindings
        // into the new scope.
        self.sym_tab.init_scope();
        match bindings {
            Some(nodes) => {
                for node in nodes {
                    match &node {
                        AstNode::PrimaryExpr { ref tkn } => {
                            let name = tkn.get_name();
                            self.sym_tab.store(&name, node.clone());
                        }
                        _ => (),
                    };
                }
            }
            None => (),
        };

        // Match on declarations until we reach the closing brace.
        loop {
            match self.curr_tkn.ty {
                TokenTy::RightBrace | TokenTy::Eof => break,
                _ => {
                    let result = self.decl()?;
                    decls.push(result);
                }
            };
        }

        // Consume closing brace and close the block scope.
        self.expect(TokenTy::RightBrace)?;
        let lvl = self.sym_tab.level();
        self.sym_tab.close_scope();

        Ok(AstNode::Block {
            decls: decls,
            scope: lvl,
        })
    }

    /// Parses a variable declaration, including optional assignment.
    ///
    /// vardecl ::= "var" IDENT [ "=" expr | arraydecl | tabledecl ] ";" ;
    fn var_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Var)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            let ty_str = self.curr_tkn.ty.to_string();
            return Err(self.add_error(ParseErrTy::ExpectedIdent(ty_str)));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        // If we now find a "=" token, we parse the assignment by declaring an array or table,
        // or continuing to expression parsing. If we encounter a ";" token intead, we store the
        // decl in the symbol table with no rhs.
        match self.curr_tkn.ty {
            TokenTy::Eq => {
                self.expect(TokenTy::Eq)?;

                let rhs = match self.curr_tkn.ty {
                    TokenTy::LeftBracket => self.array_decl(ident_tkn.clone())?,
                    TokenTy::DoubleLeftBracket => self.table_decl(ident_tkn.clone())?,
                    _ => {
                        let res = self.expr()?;
                        self.expect(TokenTy::Semicolon)?;
                        res
                    }
                };

                let node = AstNode::VarDecl {
                    ident_tkn: ident_tkn.clone(),
                    is_global: self.sym_tab.is_global(),
                    rhs: Some(Box::new(rhs)),
                };

                self.sym_tab.store(&ident_tkn.get_name(), node.clone());
                Ok(node)
            }
            TokenTy::Semicolon => {
                self.expect(TokenTy::Semicolon)?;

                let node = AstNode::VarDecl {
                    ident_tkn: ident_tkn.clone(),
                    is_global: self.sym_tab.is_global(),
                    rhs: None,
                };

                self.sym_tab.store(&ident_tkn.get_name(), node.clone());
                Ok(node)
            }
            _ => Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                "expected either '=' or ';",
            )))),
        }
    }

    /// Parse a function declaration. This function does some symbol table dancing in
    /// order to properly parse recursive function definitions.
    ///
    /// funcdecl ::= "fn" IDENT "(" [ paramlist ] ")" block ;
    fn fn_decl(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Fn)?;

        let maybe_ident_tkn = self.match_ident();
        if maybe_ident_tkn.is_none() {
            let ty_str = self.curr_tkn.ty.to_string();
            return Err(self.add_error(ParseErrTy::ExpectedIdent(ty_str)));
        }
        let ident_tkn = maybe_ident_tkn.unwrap();

        // Insert a placeholder so recursive calls pass. We store the actual
        // node later on by calling this function again.
        self.sym_tab.store(&ident_tkn.get_name(), AstNode::Null);

        self.expect(TokenTy::LeftParen)?;
        let params = self.param_list(false)?;
        self.expect(TokenTy::RightParen)?;

        // Insert function params into symtab for block parsing.
        let bindings = match &params {
            AstNode::FnParams { ref params } => {
                if params.len() > 0 {
                    Some(params.clone())
                } else {
                    None
                }
            }
            _ => None,
        };

        let body = self.block(bindings)?;

        // We don't need to write a return statement in every function, but we need
        // to insert the return ast if we don't have one so we can generate the right code
        // and jumps later when we compile.
        // TODO: we should do this in the compiler though...
        let body_w_ret = match body {
            AstNode::Block { ref decls, scope } => match decls.len() {
                0 => {
                    let mut new_decls = decls.clone();
                    new_decls.push(AstNode::ReturnStmt { ret_expr: None });
                    AstNode::Block {
                        decls: new_decls,
                        scope: scope,
                    }
                }
                _ => {
                    let last_node = &decls[decls.len() - 1];
                    match last_node {
                        AstNode::ReturnStmt { .. } => body.clone(),
                        _ => {
                            let mut new_decls = decls.clone();
                            new_decls.push(AstNode::ReturnStmt { ret_expr: None });
                            AstNode::Block {
                                decls: new_decls,
                                scope: scope,
                            }
                        }
                    }
                }
            },
            _ => body,
        };

        let node = AstNode::FnDecl {
            ident_tkn: ident_tkn.clone(),
            fn_params: Box::new(params),
            fn_body: Box::new(body_w_ret),
            scope: self.sym_tab.level(),
        };

        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    /// Processes param lists when we declare a function and when we call a function.
    /// When we declare a function, params can only be identifiers, so we do not recurse and instead
    /// match on the ident. We need to pass false in as could_be_expr in this case.
    /// When we call this from a function call expression, we need to pass true here, because
    /// our params passed in could be expressions and we need to parse them.
    ///
    /// paramlist ::= [ { IDENT [ "," ] } ] ;
    fn param_list(&mut self, could_be_expr: bool) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty {
            TokenTy::RightParen => {
                // This indicates an empty param list.
                Ok(AstNode::FnParams { params: Vec::new() })
            }
            _ => {
                // We expect the param list to be all identifiers: fn f(x, y, z). We could
                // call expr() for each param, but we can short circuit the recursive calls
                // by just creating PrimaryExpr nodes here and adding them to the param list.
                let mut param_list = Vec::new();

                while self.curr_tkn.ty != TokenTy::RightParen {
                    // Error on too many params.
                    if param_list.len() >= FN_PARAM_MAX_LEN {
                        return Err(self.add_error(ParseErrTy::FnParmCntExceeded(FN_PARAM_MAX_LEN)));
                    }

                    // Missing closing paren or unexpected end of file.
                    if self.curr_tkn.ty == TokenTy::Eof {
                        return Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                            "unexpected end of file",
                        ))));
                    }

                    if could_be_expr {
                        // Parsing a function call, where the params could be nested expressions.
                        let param = self.expr()?;
                        param_list.push(param);
                    } else {
                        // Parsing a function declaration, where the params are guaranteed to be
                        // identifiers.
                        let maybe_ident_tkn = self.match_ident();
                        if maybe_ident_tkn.is_none() {
                            let ty_str = self.curr_tkn.ty.to_string();
                            return Err(self.add_error(ParseErrTy::ExpectedIdent(ty_str)));
                        }

                        let ident_tkn = maybe_ident_tkn.unwrap();
                        param_list.push(AstNode::PrimaryExpr { tkn: ident_tkn });
                    }

                    if self.curr_tkn.ty != TokenTy::RightParen {
                        self.expect(TokenTy::Comma)?;
                    }
                }

                Ok(AstNode::FnParams { params: param_list })
            }
        }
    }

    /// Parses a table declaration.
    ///
    /// tabledecl ::= "[[" [ itemlist ] "]]" ;
    fn table_decl(&mut self, ident_tkn: Token) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::DoubleLeftBracket)?;
        let items = self.item_list()?;
        self.expect(TokenTy::DoubleRightBracket)?;

        let node = AstNode::Table {
            ident_tkn: ident_tkn.clone(),
            items: Box::new(items),
        };
        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        self.expect(TokenTy::Semicolon)?;

        Ok(node)
    }

    /// Parses a list of table items in a table declaration.
    ///
    /// itemlist ::= { IDENT "=>" expr "," } ;
    fn item_list(&mut self) -> Result<AstNode, ParseErr> {
        let mut items = HashMap::new();

        while self.curr_tkn.ty != TokenTy::DoubleRightBracket {
            // Missing closing double bracket or unexpected end of file.
            if self.curr_tkn.ty == TokenTy::Eof {
                return Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                    "unexpected end of file",
                ))));
            }

            // Get the key for the table entry.
            let maybe_ident_tkn = self.match_ident();
            if maybe_ident_tkn.is_none() {
                return Err(
                    self.add_error(ParseErrTy::InvalidTkn(String::from("expected identifier")))
                );
            }
            let ident_tkn = maybe_ident_tkn.unwrap();

            // Parse the value for this entry using expr().
            self.expect(TokenTy::EqArrow)?;
            let val = self.expr()?;
            items.insert(String::from(ident_tkn.get_name()), val);

            // Entries are separated by comma tokens. The final comma in the list is
            // optional: if we expect it but don't find it we must break. This is ok to do
            // here, as the table_decl() method will match the ending bracket token for us.
            match self.optional(TokenTy::Comma) {
                false => {
                    break;
                }
                _ => {}
            };
        }

        Ok(AstNode::ItemList { items: items })
    }

    /// Parses an array declaration.
    ///
    /// arraydecl ::= "[" { expr "," } "]" ;
    fn array_decl(&mut self, ident_tkn: Token) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::LeftBracket)?;

        let mut arr_items = Vec::new();
        while self.curr_tkn.ty != TokenTy::RightBracket {
            if self.curr_tkn.ty == TokenTy::Eof {
                return Err(self.add_error(ParseErrTy::InvalidTkn(String::from(
                    "unexpected end of file",
                ))));
            }

            let curr_item = self.expr()?;
            arr_items.push(curr_item);

            // If we have a comma here, consume and continue. If we don't
            // have one, break the loop
            match self.optional(TokenTy::Comma) {
                false => {
                    break;
                }
                _ => {}
            };
        }

        self.expect(TokenTy::RightBracket)?;
        self.expect(TokenTy::Semicolon)?;

        let len = arr_items.len();
        let box_body = match len {
            0 => None,
            _ => Some(Box::new(AstNode::ArrayItems { items: arr_items })),
        };

        let node = AstNode::Array {
            ident_tkn: ident_tkn.clone(),
            body: box_body,
            len: len,
        };
        self.sym_tab.store(&ident_tkn.get_name(), node.clone());

        Ok(node)
    }

    /// Parses an if-elif-else statement block.
    ///
    /// ifstmt ::= "if" expr block { "elif" expr block } [ "else" block ] ;
    fn if_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::If)?;

        // Parse the condition expression first, and then build the
        // AST for the statements inside the if statement.
        let if_cond = self.expr()?;
        let if_blck = self.block(None)?;

        let mut else_blck = Vec::new();
        let mut else_ifs = Vec::new();
        let mut else_cnt = 0;

        // Continue parsing until we don't match on elif or else tokens.
        loop {
            match self.curr_tkn.ty {
                TokenTy::Elif => {
                    self.expect(TokenTy::Elif)?;

                    let elif_ast = self.expr()?;
                    let elif_blck = self.block(None)?;
                    let stmt_ast = AstNode::ElifStmt {
                        cond_expr: Box::new(elif_ast),
                        stmts: Box::new(elif_blck),
                    };
                    else_ifs.push(stmt_ast);
                }
                TokenTy::Else => {
                    else_cnt = else_cnt + 1;
                    self.expect(TokenTy::Else)?;
                    let blck = self.block(None)?;
                    else_blck.push(blck);
                }
                _ => break,
            };
        }

        // Don't allow multiple else statements.
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

    /// Parses a for loop statement.
    ///
    /// forstmt ::= "for" identpair "in" expr block ;
    fn for_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::For)?;

        let var_list = self.ident_pair()?;

        self.expect(TokenTy::In)?;

        let in_expr_list = self.expr()?;

        let bindings = match &var_list {
            AstNode::IdentPair { ref idents } => Some(idents.clone()),
            _ => None,
        };

        let stmts = self.block(bindings)?;

        Ok(AstNode::ForStmt {
            var_list: Box::new(var_list),
            in_expr_list: Box::new(in_expr_list),
            stmts: Box::new(stmts),
        })
    }

    /// Parses a pair of identifiers. This is intended to be used inside the for loop, but
    /// could also be re-used if any tuple structures need to be parsed.
    ///
    /// identpair ::= IDENT "," IDENT ;
    fn ident_pair(&mut self) -> Result<AstNode, ParseErr> {
        let mut idents = Vec::new();

        let maybe_first_ident = self.match_ident();
        if maybe_first_ident.is_none() {
            let ty_str = self.curr_tkn.ty.to_string();
            return Err(self.add_error(ParseErrTy::ExpectedIdent(ty_str)));
        }
        let first_ident = maybe_first_ident.unwrap();
        idents.push(AstNode::PrimaryExpr { tkn: first_ident });

        self.expect(TokenTy::Comma)?;

        let maybe_sec_ident = self.match_ident();
        if maybe_sec_ident.is_none() {
            let ty_str = self.curr_tkn.ty.to_string();
            return Err(self.add_error(ParseErrTy::ExpectedIdent(ty_str)));
        }
        let second_ident = maybe_sec_ident.unwrap();
        idents.push(AstNode::PrimaryExpr { tkn: second_ident });

        Ok(AstNode::IdentPair { idents: idents })
    }

    /// Parses a return statement.
    ///
    /// retstmt ::= "return" [ expr ] ";" ;
    fn ret_stmt(&mut self) -> Result<AstNode, ParseErr> {
        self.expect(TokenTy::Return)?;

        match self.curr_tkn.ty {
            // With no rhs expr, put None into the resulting AST node.
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

    /// Parses an expression statement. This isn't strictly needed, but allows for some easier
    /// recursion in the parser. When we need to recurse and the next AST node could be a statement
    /// or an expression, we can call stmt(), and if we find an expression we can use this method.
    /// This allows us to avoid matching on tokens in multiple places to determine whether to call
    /// stmt() or expr(). All this does is parse an expression.
    ///
    /// exprstmt ::= expr ";" ;
    fn expr_stmt(&mut self) -> Result<AstNode, ParseErr> {
        let node = self.expr()?;
        self.expect(TokenTy::Semicolon)?;
        Ok(AstNode::ExprStmt {
            expr: Box::new(node),
        })
    }

    /// Parses an expression. Because the grammar encodes precedence, we must call each
    /// expression parsing method in the order starting from most general (lowest precedence)
    /// to most specific (highest precedence). Note that we recurse to higher precedence
    /// expressions first in each expression parse method, which is why we start with
    /// the lowest precedence instead of the highest.
    ///
    /// Precedence roughly follows this table:
    /// | = Assignment       | <- Lowest precedence
    /// | ||                 |
    /// | &&                 |
    /// | ==, !=             |
    /// | >, <, >=, <=       |
    /// | +, -               |
    /// | *, /               |
    /// | %                  |
    /// | -, ! Unary         |
    /// | Literals           | <- Highest precedence
    ///
    /// expr ::= assignexpr ;
    fn expr(&mut self) -> Result<AstNode, ParseErr> {
        self.assign_expr()
    }

    /// Parses an assign expression.
    ///
    /// assignexpr ::= { [ funccall "." ] [ arrayaccess ] assignexpr } | orexpr ;
    fn assign_expr(&mut self) -> Result<AstNode, ParseErr> {
        let ast = self.or_expr()?;

        match self.curr_tkn.ty {
            TokenTy::Eq => {
                let op = self.curr_tkn.clone();
                self.expect(TokenTy::Eq)?;

                // Get the assignment value
                let rhs = self.assign_expr()?;

                // Check the lhs of the expression. If it's an ident, we have a variable assignment.
                // We check the symbol table for that variable, and error if we can't find it.
                // If the lhs is an array access, we're mutating an array value.
                // If it's neither of those, we have an invalid assignment.
                match ast.clone() {
                    AstNode::PrimaryExpr { tkn } => {
                        match tkn.ty {
                            TokenTy::Ident(name) => {
                                let maybe_sym = self.sym_tab.retrieve(&name);
                                if maybe_sym.is_none() {
                                    return Err(self.add_error(ParseErrTy::UndeclSym(name)));
                                }

                                // Check symbol table for var name.
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
                    AstNode::ArrayAccess { array_tkn, index } => {
                        return Ok(AstNode::ArrayMutExpr {
                            array_tkn: array_tkn,
                            index: index,
                            rhs: Box::new(rhs),
                        });
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

    /// Parses an or ("||") expression.
    ///
    /// orexpr ::= andexpr { [ "||" ] andexpr } ;
    fn or_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.and_expr()?;

        // Continue parsing as long as the rhs contains expressions.
        loop {
            match self.curr_tkn.ty {
                TokenTy::PipePipe => {
                    let op = self.curr_tkn.clone();
                    self.consume();
                    let rhs = self.and_expr()?;

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

    /// Parses an and ("&&") expression.
    ///
    /// andexpr ::= eqexpr { [ "&&" ] eqexpr } ;
    fn and_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.equality_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::AmpAmp => {
                    let op = self.curr_tkn.clone();
                    self.consume();
                    let rhs = self.equality_expr()?;

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

    /// Parses an equality ("!=" and "==") expression.
    ///
    /// eqexpr ::= cmpexpr { [ "!=" ] [ "==" ] cmpexpr } ;
    fn equality_expr(&mut self) -> Result<AstNode, ParseErr> {
        let mut ast = self.compr_expr()?;

        loop {
            match self.curr_tkn.ty {
                TokenTy::BangEq | TokenTy::EqEq => {
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

    /// Parses a comparison (">", "<", ">=", "<=") expression.
    ///
    /// cmpexpr ::= addorsubexpr { [ ">" ] [ ">=" ] [ "<" ] [ "<=" ] addorsubexpr } ;
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

    /// Parses addition or subtraction expressions.
    ///
    /// addorsubexpr ::= mulordivexpr { [ "+" ] [ "-" ] mulordivexpr } ;
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

    /// Parses a multiplaction or division expression.
    ///
    /// mulordivexpr ::= modexpr { [ "*" ] [ "/" ] modexpr } ;
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

    /// Parses a modulo expression.
    ///
    /// modexpr ::= unaryexpr { [ "%" ] unaryexpr } ;
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

    /// Parses a unary expression. Unary's include number negation and logical negation.
    ///
    /// unaryexpr ::= [ "-" ]  [ "!" ] unaryexpr | funccall ;
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
            _ => self.fn_call_expr(),
        }
    }

    /// Parse a function call expression. Function calls are expression because they return a
    /// value: whatever the function returns. There is some matching involved here to determine
    /// whether or not we're calling a stdlib function, because stdlib functions aren't inserted into
    /// the symbol table (otherwise we can't declare functions of the same name).
    ///
    /// funccall ::= [ "@" ] primary "(" [ paramlist ] ")" |
    ///              tableaccess |
    ///		     arrayaccess |
    ///		     primary ;
    fn fn_call_expr(&mut self) -> Result<AstNode, ParseErr> {
        let ast = self.primary_expr()?;
        let mut params = Vec::new();
        let ident_tkn = match ast {
            AstNode::PrimaryExpr { ref tkn } => Some(tkn.clone()),
            _ => None,
        };

        match self.curr_tkn.ty {
            TokenTy::LeftParen => {
                self.expect(TokenTy::LeftParen)?;
                // This is a function call, so pass true into param_list(), indicating we
                // need to parse the params as possible expressions.
                let params_list = self.param_list(true)?;
                self.expect(TokenTy::RightParen)?;
                match params_list {
                    AstNode::FnParams {
                        params: inner_params,
                    } => {
                        params = inner_params;
                    }
                    _ => (),
                };

                let ident_name = ident_tkn.clone().unwrap().get_name();
                let maybe_ast = self.sym_tab.retrieve(&ident_name);
                let is_std = crate::reserved::is_std_lib_fn(&ident_name);

                // If we can't find the function name in the ast, we assume it's undeclared,
                // GIVEN that the symbol is not a standard lib function.
                if maybe_ast.is_none() {
                    if !is_std {
                        let err = self.add_error(ParseErrTy::UndeclSym(ident_name.to_string()));
                        self.consume();
                        return Err(err);
                    }
                }

                // Used to check if this may be a recursive call. If it is, we skip some
                // further checks and assume the function will be defined properly in
                // the symbol table after further parsing. If there are any errors they will get
                // raised at runtime or possibly compile time.
                let is_null = (!is_std)
                    && match maybe_ast.clone().unwrap() {
                        AstNode::Null => true,
                        _ => false,
                    };

                // HACK: if not standard, we match the params and error on the wrong param
                // count. If it is standard lib, just skip for now
                if !is_std && !is_null {
                    let expected_param_len = match maybe_ast.unwrap() {
                        AstNode::FnDecl {
                            ident_tkn: _,
                            fn_params,
                            ..
                        } => match *fn_params {
                            AstNode::FnParams { params } => params.len(),
                            _ => 0,
                        },
                        _ => 0,
                    };

                    if params.len() != expected_param_len {
                        let err = self.add_error(ParseErrTy::WrongFnParmCnt(
                            expected_param_len,
                            params.len(),
                        ));
                        return Err(err);
                    }
                }

                return Ok(AstNode::FnCallExpr {
                    fn_ident_tkn: ident_tkn.unwrap(),
                    fn_params: params,
                    is_std: is_std,
                });
            }
            TokenTy::Period => {
                self.expect(TokenTy::Period)?;
                self.should_check_sym_tab = false;
                let val = self.expr()?;
                self.should_check_sym_tab = true;
                return Ok(AstNode::TableAccess {
                    table_tkn: ident_tkn.unwrap(),
                    index: Box::new(val),
                });
            }
            TokenTy::LeftBracket => {
                self.expect(TokenTy::LeftBracket)?;
                let idx = self.expr()?;
                self.expect(TokenTy::RightBracket)?;
                return Ok(AstNode::ArrayAccess {
                    array_tkn: ident_tkn.unwrap(),
                    index: Box::new(idx),
                });
            }
            _ => (),
        };

        Ok(ast)
    }

    /// Parse a primary expression. Primary refers to either primitive types/values. This roughly
    /// corresponds to:
    /// 1. Number literals
    /// 2. String literals
    /// 3. Boolean literals
    /// 4. Identifiers
    /// 5. Parens, indicating a grouped expression.
    ///
    /// primary  ::= NUMBER |
    ///              STRING |
    ///              TRUE   |
    ///              FALSE  |
    ///              IDENT  |
    ///              groupexpr ;
    fn primary_expr(&mut self) -> Result<AstNode, ParseErr> {
        match self.curr_tkn.ty.clone() {
            TokenTy::Str(_) | TokenTy::Val(_) | TokenTy::True | TokenTy::False => {
                // Number/string/boolean literal.
                let ast = Ok(AstNode::PrimaryExpr {
                    tkn: self.curr_tkn.clone(),
                });
                self.consume();
                ast
            }
            TokenTy::Ident(ref ident_name) => {
                // Identifier. If required, check the symbol table to see if this identifier
                // exists. It may not be necessary if we're declaring a table.
                let ident_tkn = self.curr_tkn.clone();

                if self.should_check_sym_tab {
                    if !self.sym_exists(ident_name) {
                        if !crate::reserved::is_std_lib_fn(ident_name) {
                            let err = self.add_error(ParseErrTy::UndeclSym(ident_name.to_string()));
                            self.consume();
                            return Err(err);
                        }
                    }
                }

                let ast = Ok(AstNode::PrimaryExpr {
                    tkn: ident_tkn.clone(),
                });

                self.consume();
                ast
            }
            TokenTy::LeftParen => self.group_expr(),
            TokenTy::At => {
                // Stdlib function call.
                self.expect(TokenTy::At)?;
                return self.fn_call_expr();
            }
            _ => {
                let ty_str = self.curr_tkn.ty.to_string();
                let err = self.add_error(ParseErrTy::InvalidTkn(ty_str));
                self.consume();
                Err(err)
            }
        }
    }

    /// Parses an expression surrounded by parenthesis.
    ///
    /// groupexpr ::= "(" expr ")" ;
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

    /// Checks that the token matches what we expect. If it does, we consume it and return true.
    /// If not, return false. Note that we do not report any error here, since the token we
    /// expect is considered optional. The return value here should be used to what function the
    /// parser should move to next.
    /// For example, when parsing an array declaration, we can accept either of the following:
    ///
    /// var a = [
    ///   1,
    ///   2,
    ///   3,
    /// ]
    /// var b = [1,2,3]
    ///
    /// If we call `expect(TokenTy::Comma)` after parsing the value 3, the parser will error.
    /// If we don't call `expect(TokenTy::Comma)`, we don't support the trailing comma syntax.
    /// Instead, we can call this function and then choose to match on the RightParen if this
    /// returns false.
    fn optional(&mut self, tknty: TokenTy) -> bool {
        if self.curr_tkn.ty == tknty {
            self.consume();
            return true;
        }
        false
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

    /// Advance to the next token, discarding the previously read token.
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

    fn sym_exists(&self, key: &str) -> bool {
        self.sym_tab.contains(key)
    }
}
