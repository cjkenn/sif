extern crate clap;

extern crate sif_parse;

use clap::Clap;

use sif_parse::{
    lex::Lexer,
    parser::{Parser, ParserResult},
    symbol_table::SymTab,
};

use std::{fs::File, io};

#[derive(Clap)]
#[clap(version = "1.0")]
pub struct SifOpts {
    #[clap(long)]
    filename: Option<String>,
    #[clap(long)]
    dump_ast: bool,
}

fn main() {
    let opts: SifOpts = SifOpts::parse();

    match &opts.filename {
        Some(_) => from_file(opts),
        None => repl(),
    };
}

fn from_file(opts: SifOpts) {
    // 1. Run the lexer/parser.
    let mut symtab = SymTab::new();
    let parse_result = run_parser(&opts.filename.unwrap(), &mut symtab);

    // Any errors should already have been emitted by the
    // parser, whether or not they are continuable.
    if parse_result.has_err {
        println!("sif: Exiting due to parser errors");
        return;
    }

    if opts.dump_ast {
        let ast = parse_result.ast.unwrap();
        println!("{:#?}", ast);
    }

    // 2. Convert AST to chunks

    // 3. Start vm and interpret chunks
}

fn repl() {
    let mut _symtab = SymTab::new();
    let mut input = String::new();

    // TODO: how do we lex from a stdin input string here?
    println!("Welcome to sif!");
    print!(">");
    match io::stdin().read_line(&mut input) {
        Ok(_) => println!("{:#?}", input),
        Err(e) => panic!(e),
    };
}

/// Opens the file from the filename provided, creates a lexer for that file
/// and a parser for that lexer. Fully parses the input file, and returns
/// the result from the parser. This result will contain any errors, as well
/// as the AST from parsing (which will be None if there are errors).
fn run_parser(filename: &str, symtab: &mut SymTab) -> ParserResult {
    let infile = match File::open(filename) {
        Ok(file) => file,
        Err(e) => {
            panic!("sif: could not open file '{}': {:?}", filename, e.kind());
        }
    };

    let mut lexer = Lexer::new(infile);
    let mut parser = Parser::new(&mut lexer, symtab);
    parser.parse()
}
