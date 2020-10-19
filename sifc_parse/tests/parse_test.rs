use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};

use std::fs::File;

mod util;

#[test]
fn if_stmt() {
    let pctx = util::setup("if_stmt");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn for_stmt() {
    let pctx = util::setup("for_stmt");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn var_decl_valid() {
    let pctx = util::setup("var_decl");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn var_decl_invalid() {
    let pctx = util::setup("var_decl_invalid");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_decl_valid() {
    let pctx = util::setup("fn_decl_valid");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_decl_invalid() {
    let pctx = util::setup("fn_decl_invalid");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_decl_wrong_params() {
    let pctx = util::setup("fn_decl_wrong_params");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_w_ret_stmt() {
    let pctx = util::setup("fn_w_ret_stmt");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_call() {
    let pctx = util::setup("fn_call");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn fn_call_wrong_params() {
    let pctx = util::setup("fn_call_wrong_params");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn exprs() {
    let pctx = util::setup("exprs");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn table_decl() {
    let pctx = util::setup("table_decl");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn record_decl() {
    let pctx = util::setup("record_decl");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}

#[test]
fn array_decl() {
    let pctx = util::setup("array_decl");

    let infile = File::open(&pctx.path).unwrap();
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(infile);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let result = parser.parse();
    util::check(pctx, result);
}
