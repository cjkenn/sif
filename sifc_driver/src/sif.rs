extern crate clap;
extern crate sifc_compiler;
extern crate sifc_parse;

use clap::Clap;
use sifc_compiler::{
    compiler::{CompileResult, Compiler},
    dreg::DReg,
    printer,
};
use sifc_err::err::SifErr;
use sifc_parse::{
    ast::AstNode,
    lex::Lexer,
    parser::{Parser, ParserResult},
    symtab::SymTab,
};
use std::{cell::RefCell, fs::File, io, rc::Rc};

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
    // 1. Run the lexer/parser.
    let mut symtab = SymTab::new();
    let parse_result = run_parser(&opts.filename.unwrap(), &mut symtab);

    // Any errors should already have been emitted by the
    // parser, whether or not they are continuable.
    if parse_result.has_err {
        println!("sif: Exiting due to parser errors");
        return;
    }

    let ast = parse_result.ast.unwrap();

    if opts.dump_ast {
        println!("{:#?}", ast);
    }

    // 2. Convert AST to instructions
    let comp_result = run_compiler(&ast);
    match comp_result {
        Err(e) => {
            e.emit();
            return;
        }
        _ => (),
    };

    if opts.dump_ir {
        //println!("{:#?}", comp_result.unwrap());
        printer::dump(comp_result.unwrap());
    }

    // 3. Start vm and interpret instruction blocks
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

fn run_compiler(ast: &AstNode) -> CompileResult {
    // Init data register array
    let mut regs = Vec::with_capacity(1024);
    for i in 0..1023 {
        let reg = DReg::new(format!("r{}", i));
        regs.push(Rc::new(RefCell::new(reg)));
    }

    let mut comp = Compiler::new(ast, regs);
    comp.compile()
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
