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
use sifc_vm::{config::VMConfig, vm::VM};
use std::{fs::File, io, time::Instant};

// Default size of heap, in number of items, NOT bytes.
const HEAP_INIT_ITEMS: usize = 100;

// Default size of data register vec. If we exceeed this len,
// we can increase the size of the vec.
const DREG_INITIAL_LEN: usize = 64;

#[derive(Clap)]
#[clap(version = "1.0")]
pub struct SifOpts {
    #[clap(long)]
    filename: Option<String>,
    #[clap(long)]
    print_ast: bool,
    #[clap(long)]
    print_ir: bool,
    #[clap(long)]
    trace: bool,
}

fn main() {
    let opts: SifOpts = SifOpts::parse();

    match &opts.filename {
        Some(_) => from_file(opts),
        None => repl(),
    };
}

fn from_file(opts: SifOpts) {
    let exec_start = Instant::now();

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

    if opts.print_ast {
        println!("{:#?}", ast);
    }

    compile_and_run(opts, &ast);
    println!("sif: execution completed in {:#?}", exec_start.elapsed());
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

    println!("sif: parsing file {}...", filename);
    let mut lexer = Lexer::new(infile);
    let mut parser = Parser::new(&mut lexer, symtab);
    parser.parse()
}

fn compile_and_run(opts: SifOpts, ast: &AstNode) {
    println!("sif: compiling...");
    let mut comp = Compiler::new(ast);
    let comp_result = comp.compile();

    let maybe_err = comp_result.err;
    if maybe_err.is_some() {
        maybe_err.unwrap().emit();
        eprintln!("sif: exiting due to errors");
        return;
    }

    let program = comp_result.program;
    let code_start = comp_result.code_start;
    let jumptab = comp_result.jumptab;
    let fntab = comp_result.fntab;

    // TODO: add option to write to file and not run vm?
    if opts.print_ir {
        printer::dump_decls(comp_result.decls.clone());
        printer::dump_code(comp_result.code.clone());
    }

    println!("sif: starting vm...");
    if opts.trace {
        println!("sif: tracing vm execution!\n");
    }

    let conf = VMConfig {
        trace: opts.trace,
        initial_heap_size: HEAP_INIT_ITEMS,
        initial_dreg_count: DREG_INITIAL_LEN,
    };

    // TODO: use a param struct for this? A builder?
    let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
    let vm_result = vm.run();
    match vm_result {
        Ok(()) => {}
        Err(e) => {
            e.emit();
            eprintln!("sif: exiting due to errors");
            return;
        }
    }
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
