extern crate clap;
extern crate sifc_compiler;
extern crate sifc_parse;
extern crate sifc_vm;

use clap::Clap;

use sifc_compiler::{compiler::Compiler, printer};

use sifc_err::err::SifErr;

use sifc_parse::{
    ast::AstNode,
    lex::Lexer,
    parser::{Parser, ParserResult},
    symtab::SymTab,
};

use sifc_vm::vm::VM;

use std::{fs::File, io};

#[derive(Clap)]
#[clap(version = "1.0")]
pub struct SifOpts {
    #[clap(long)]
    filename: Option<String>,
    #[clap(long)]
    dump_ast: bool,
    #[clap(long)]
    dump_ir: bool,
}

fn main() {
    let opts: SifOpts = SifOpts::parse();

    match &opts.filename {
        Some(_) => from_file(opts),
        None => repl(),
    };
}

fn from_file(opts: SifOpts) {
    let mut symtab = SymTab::new();
    let path = opts.filename.clone().unwrap();
    let parse_result = parse(&path, &mut symtab);

    // Any errors should already have been emitted by the
    // parser, whether or not they are continuable.
    if parse_result.has_err {
        eprintln!("sif: Exiting due to parser errors");
        return;
    }

    let ast = parse_result.ast.unwrap();

    if opts.dump_ast {
        println!("{:#?}", ast);
    }

    compile_and_run(opts, &ast, &symtab);
}

/// Opens the file from the filename provided, creates a lexer for that file
/// and a parser for that lexer. Fully parses the input file, and returns
/// the result from the parser. This result will contain any errors, as well
/// as the AST from parsing (which will be None if there are errors).
fn parse(filename: &str, symtab: &mut SymTab) -> ParserResult {
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

fn compile_and_run(opts: SifOpts, ast: &AstNode, symtab: &SymTab) {
    let mut comp = Compiler::new(ast, symtab);
    let comp_result = comp.compile();
    match comp_result {
        Err(e) => {
            e.emit();
            eprintln!("sif: exiting due to errors");
            return;
        }
        _ => (),
    };

    let code = comp_result.unwrap();

    // TODO: add option to write to file and not run vm?
    if opts.dump_ir {
        printer::dump(code.clone());
    }

    let mut vm = VM::new(code);
    vm.run();
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
