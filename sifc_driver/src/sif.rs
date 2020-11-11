extern crate clap;
extern crate sifc_bytecode;
extern crate sifc_parse;
extern crate sifc_vm;

mod timings;

use crate::timings::Timings;

use clap::{App, Arg, ArgMatches};
use sifc_bytecode::{
    compiler::{CompileResult, Compiler},
    optimize::bco::{BytecodeOptimizer, OptimizeResult},
    printer,
};
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

// Default size of data register vec. If we exceed this len,
// we can increase the size of the vec.
const DEFAULT_DREG: &str = "1024";

const ARG_FILENAME: &str = "filename";
const ARG_EMIT_AST: &str = "emit-ast";
const ARG_EMIT_IR: &str = "emit-ir";
const ARG_TRACE_EXEC: &str = "trace-exec";
const ARG_HEAP_SIZE: &str = "heap-size";
const ARG_REG_COUNT: &str = "reg-count";
const ARG_DUR: &str = "timings";
const ARG_BC_OPT: &str = "bco";

fn main() {
    let matches = parse_cl();
    from_file(matches);
}

fn from_file(opts: ArgMatches) {
    let exec_start = Instant::now();

    let mut timings: Timings = Default::default();
    let show_duration = opts.is_present(ARG_DUR);
    let mut symtab = SymTab::new();
    let path = opts.value_of(ARG_FILENAME).unwrap();

    let parse_result = parse(&path, &mut symtab);
    timings.parse_time = exec_start.elapsed();

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

    let compile_start = Instant::now();
    let comp_result = compile(&ast);
    timings.compile_time = compile_start.elapsed();

    let maybe_err = &comp_result.err;
    if maybe_err.is_some() {
        maybe_err.as_ref().unwrap().emit();
        eprintln!("sif: exiting due to errors");
        return;
    }

    if opts.is_present(ARG_EMIT_IR) {
        printer::dump_decls(comp_result.decls.clone());
        printer::dump_code(comp_result.code.clone());
    }

    if opts.is_present(ARG_BC_OPT) {
        let opt_start = Instant::now();
        let opt_result = run_optimizer(&comp_result);
        timings.optimize_time = opt_start.elapsed();

        let vm_start = Instant::now();
        // TODO: need to provide better params/options to run_vm method
        run_vm_optimized(opts, opt_result);
        timings.vm_time = vm_start.elapsed();
    } else {
        let vm_start = Instant::now();
        run_vm_raw(opts, comp_result);
        timings.vm_time = vm_start.elapsed();
    }

    timings.total_time = exec_start.elapsed();

    if show_duration {
        timings.emit();
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

fn compile(ast: &AstNode) -> CompileResult {
    let mut comp = Compiler::new(ast);
    comp.compile()
}

fn run_optimizer(comp_result: &CompileResult) -> OptimizeResult {
    let mut opt = BytecodeOptimizer::new(
        comp_result.decls.clone(),
        comp_result.code.clone(),
        comp_result.code_start,
    );

    opt.run_passes()
}

fn run_vm_optimized(opts: ArgMatches, opt_result: OptimizeResult) {
    let program = opt_result.optimized;

    let code_start = opt_result.new_code_start;
    let jumptab = opt_result.jumptab;
    let fntab = opt_result.fntab;
    let heap_size: usize = opts.value_of(ARG_HEAP_SIZE).unwrap().parse().unwrap();
    let dreg_count: usize = opts.value_of(ARG_REG_COUNT).unwrap().parse().unwrap();

    let conf = VMConfig {
        trace: opts.is_present(ARG_TRACE_EXEC),
        initial_heap_size: heap_size,
        initial_dreg_count: dreg_count,
    };

    // TODO: use a param struct for this? A builder?
    let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
    let vm_result = vm.run();

    match vm_result {
        Ok(()) => {}
        Err(e) => {
            e.emit();
            eprintln!("sif: exiting due to errors");
        }
    }
}

fn run_vm_raw(opts: ArgMatches, comp_result: CompileResult) {
    let program = comp_result.program;
    let code_start = comp_result.code_start;
    let jumptab = comp_result.jumptab;
    let fntab = comp_result.fntab;
    let heap_size: usize = opts.value_of(ARG_HEAP_SIZE).unwrap().parse().unwrap();
    let dreg_count: usize = opts.value_of(ARG_REG_COUNT).unwrap().parse().unwrap();

    let conf = VMConfig {
        trace: opts.is_present(ARG_TRACE_EXEC),
        initial_heap_size: heap_size,
        initial_dreg_count: dreg_count,
    };

    // TODO: use a param struct for this? A builder?
    let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
    let vm_result = vm.run();

    match vm_result {
        Ok(()) => {}
        Err(e) => {
            e.emit();
            eprintln!("sif: exiting due to errors");
        }
    }
}

fn parse_cl() -> ArgMatches {
    App::new("sif")
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
            Arg::new(ARG_DUR)
                .long(ARG_DUR)
                .about("Display basic durations for phases of sif"),
        )
        .arg(
            Arg::new(ARG_BC_OPT)
                .long(ARG_BC_OPT)
                .about("Runs the bytecode optimizer before executing in vm"),
        )
        .get_matches()
}
