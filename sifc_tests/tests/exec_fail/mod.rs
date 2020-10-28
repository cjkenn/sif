use sifc_compiler::compiler::Compiler;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};
use std::fs::File;

macro_rules! exec_fail_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            // Open input file, which is a sif program.
            let path = format!("./tests/exec_fail/inputs/{}.sif", $file_name);

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

            // Execute bytecode, ensuring a runtime error occurs
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
            assert!(vm_result.is_err());
        }
    };
}

exec_fail_test!(null_access, "null_access");
exec_fail_test!(invalid_var_access, "invalid_var_access");
exec_fail_test!(invalid_binarg, "invalid_binarg");
exec_fail_test!(binop_str_num, "binop_str_num");
exec_fail_test!(binop_str_bool, "binop_str_bool");
exec_fail_test!(binop_num_bool, "binop_num_bool");
