use sif_parse::{lex::Lexer, parser::Parser, symbol_table::SymTab};

use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

struct ParseExpect {
    pub is_pass: bool,
    pub line: usize,
    pub pos: usize,
}

/// Returns a listing of test names to their file locations.
fn get_test_path(test_name: &str) -> &str {
    let paths: HashMap<&str, &str> = [
        ("if_stmt", "./input/if_stmt.sif"),
        ("for_stmt", "./input/for_stmt.sif"),
        ("var_decl", "./input/var_decl.sif"),
    ]
    .iter()
    .cloned()
    .collect();

    let path = paths.get(test_name);
    match path {
        Some(p) => return p,
        None => panic!("invalid parser test name provided!"),
    };
}

#[test]
fn test_parse_if_stmt() {
    let path = get_test_path("if_stmt");
    let file = File::open(path).unwrap();

    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(file);
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let parse_result = parser.parse();
}
