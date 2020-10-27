use sifc_compiler::compiler::Compiler;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};
use std::fs::File;

macro_rules! exec_pass_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            // Open input file, which is a sif program.
            let path = format!("./tests/exec_pass/inputs/{}.sif", $file_name);

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

exec_pass_test!(var_decl, "var_decl");
exec_pass_test!(array_decl, "array_decl");
exec_pass_test!(exprs, "exprs");
exec_pass_test!(fn_call, "fn_call");
exec_pass_test!(fn_decl_valid, "fn_decl_valid");
exec_pass_test!(fn_w_ret_stmt, "fn_w_ret_stmt");
exec_pass_test!(for_stmt, "for_stmt");
exec_pass_test!(if_stmt, "if_stmt");
exec_pass_test!(table_decl, "table_decl");
exec_pass_test!(std_lib_calls, "std_lib_calls");
exec_pass_test!(scopes, "scopes");
