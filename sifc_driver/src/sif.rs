extern crate clap;
extern crate sifc_compiler;
extern crate sifc_parse;
extern crate sifc_vm;

use clap::{App, Arg, ArgMatches};
use sifc_compiler::{compiler::Compiler, printer};
use sifc_err::err::SifErr;
use sifc_parse::{
    ast::AstNode,
    lex::Lexer,
    parser::{Parser, ParserResult},
    symtab::SymTab,
};
use sifc_vm::{config::VMConfig, vm::VM};
use std::{fs::File, time::Instant};

// Default size of heap, in number of items, NOT bytes.
const DEFAULT_HEAP: &str = "100";

// Default size of data register vec. If we exceeed this len,
// we can increase the size of the vec.
const DEFAULT_DREG: &str = "64";

const ARG_FILENAME: &str = "filename";
const ARG_EMIT_AST: &str = "emit-ast";
const ARG_EMIT_IR: &str = "emit-ir";
const ARG_TRACE_EXEC: &str = "trace-exec";
const ARG_HEAP_SIZE: &str = "heap-size";
const ARG_REG_COUNT: &str = "reg-count";
const ARG_BENCH: &str = "bench";

fn main() {
    let matches = App::new("sif")
        .version("0.1")
        .author("cjkenn")
        .about("sif interpreter and vm")
        .arg(
            Arg::new(ARG_FILENAME)
                .about("sif file to parse and run")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new(ARG_EMIT_AST)
                .long(ARG_EMIT_AST)
                .about("Prints the syntax tree to stdout"),
        )
        .arg(
            Arg::new(ARG_EMIT_IR)
                .long(ARG_EMIT_IR)
                .about("Prints sif bytecode to stdout"),
        )
        .arg(
            Arg::new(ARG_TRACE_EXEC)
                .short('t')
                .long(ARG_TRACE_EXEC)
                .about("Traces VM execution by printing running instructions to stdout"),
        )
        .arg(
            Arg::new(ARG_HEAP_SIZE)
                .short('H')
                .long(ARG_HEAP_SIZE)
                .default_value(DEFAULT_HEAP)
                .about("Sets initial heap size"),
        )
        .arg(
            Arg::new(ARG_REG_COUNT)
                .short('R')
                .long(ARG_REG_COUNT)
                .default_value(DEFAULT_DREG)
                .about("Sets the default virtual register count"),
        )
        .arg(
            Arg::new(ARG_BENCH)
                .short('b')
                .long(ARG_BENCH)
                .about("Display basic benchmarks for phases of sif"),
        )
        .get_matches();

    from_file(matches);
}

fn from_file(opts: ArgMatches) {
    let exec_start = Instant::now();
    let is_bench = opts.is_present(ARG_BENCH);

    let mut symtab = SymTab::new();
    let path = opts.value_of(ARG_FILENAME).unwrap();
    let parse_result = parse(&path, &mut symtab);
    if is_bench {
        println!(
            "sif: parsing of '{}' completed in {:#?}",
            path,
            exec_start.elapsed()
        );
    }

    // Any errors should already have been emitted by the
    // parser, whether or not they are continuable.
    if parse_result.has_err {
        eprintln!("sif: Exiting due to parser errors");
        return;
    }

    let ast = parse_result.ast.unwrap();

    if opts.is_present(ARG_EMIT_AST) {
        println!("{:#?}", ast);
    }

    compile_and_run(opts, &ast);

    if is_bench {
        println!(
            "\nsif: total execution completed in {:#?}",
            exec_start.elapsed()
        );
    }
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

fn compile_and_run(opts: ArgMatches, ast: &AstNode) {
    let compile_start = Instant::now();

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

    if opts.is_present(ARG_EMIT_IR) {
        printer::dump_decls(comp_result.decls.clone());
        printer::dump_code(comp_result.code.clone());
    }

    let is_bench = opts.is_present(ARG_BENCH);

    if is_bench {
        println!(
            "sif: bytecode compilation completed in {:#?}",
            compile_start.elapsed()
        );
    }

    let heap_size: usize = opts.value_of(ARG_HEAP_SIZE).unwrap().parse().unwrap();
    let dreg_count: usize = opts.value_of(ARG_REG_COUNT).unwrap().parse().unwrap();

    let conf = VMConfig {
        trace: opts.is_present(ARG_TRACE_EXEC),
        initial_heap_size: heap_size,
        initial_dreg_count: dreg_count,
    };

    let vm_start = Instant::now();

    // TODO: use a param struct for this? A builder?
    let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
    let vm_result = vm.run();

    if is_bench {
        println!("sif: vm execution finished in {:#?}", vm_start.elapsed());
    }

    match vm_result {
        Ok(()) => {}
        Err(e) => {
            e.emit();
            eprintln!("sif: exiting due to errors");
            return;
        }
    }
}
