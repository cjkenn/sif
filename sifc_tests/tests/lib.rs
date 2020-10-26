extern crate sifc_compiler;
extern crate sifc_err;
extern crate sifc_parse;
extern crate sifc_vm;

use sifc_compiler::compiler::Compiler;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};

use std::fs::File;

macro_rules! integ_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            // Open input file, which is a sif program.
            let path = format!("./tests/inputs/{}.sif", $file_name);

            // Lex and parse the file, ensuring no errors.
            let infile = File::open(path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let parse_result = parser.parse();
            assert_eq!(parse_result.has_err, false);

            // Compile to bytecode, ensuring no errors.
            let ast = parse_result.ast.unwrap();
            let mut compiler = Compiler::new(&ast);
            let compile_result = compiler.compile();
            assert!(compile_result.err.is_none());

            // Execute bytecode, ensuring no panics/runtime errors.
            let conf = VMConfig {
                trace: false,
                initial_heap_size: 10,
                initial_dreg_count: 32,
            };
            let mut vm = VM::init(
                compile_result.program,
                compile_result.code_start,
                compile_result.jumptab,
                compile_result.fntab,
                conf,
            );
            let vm_result = vm.run();
            assert!(vm_result.is_ok());
        }
    };
}

integ_test!(var_decl_integ, "var_decl");
integ_test!(array_decl_integ, "array_decl");
integ_test!(exprs_integ, "exprs");
integ_test!(fn_call_integ, "fn_call");
integ_test!(fn_decl_valid_integ, "fn_decl_valid");
integ_test!(fn_w_ret_stmt_integ, "fn_w_ret_stmt");
integ_test!(for_stmt_integ, "for_stmt");
integ_test!(if_stmt_integ, "if_stmt");
integ_test!(table_decl_integ, "table_decl");
integ_test!(std_lib_calls_integ, "std_lib_calls");
integ_test!(scopes, "scopes");
