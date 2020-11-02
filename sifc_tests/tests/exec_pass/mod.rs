use sifc_bytecode::compiler::Compiler;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};
use std::fs::File;

const INPUT_PATH: &str = "./tests/exec_pass/inputs";

macro_rules! exec_pass_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            // Open input file, which is a sif program.
            let path = format!("{}/{}.sif", INPUT_PATH, stringify!($test_name));

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

exec_pass_test!(var_decl);
exec_pass_test!(array_decl);
exec_pass_test!(exprs);
exec_pass_test!(fn_call);
exec_pass_test!(fn_decl_valid);
exec_pass_test!(fn_w_ret_stmt);
exec_pass_test!(for_stmt);
exec_pass_test!(if_stmt);
exec_pass_test!(table_decl);
exec_pass_test!(std_lib_calls);
exec_pass_test!(scopes);
exec_pass_test!(array_mut_assign);
