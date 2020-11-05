use sifc_bytecode::{compiler::Compiler, sifv::SifVal};
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};

// Expects a sif program str as input, returns a vm after running and asserting the
// run completes successfully.
macro_rules! vm_run {
    ($input:expr) => {{
        let mut symtab = SymTab::new();
        let mut lex = Lexer::new($input.as_bytes());
        let mut parser = Parser::new(&mut lex, &mut symtab);

        let parse_result = parser.parse();
        assert_eq!(parse_result.has_err, false);

        let ast = parse_result.ast.unwrap();
        let mut compiler = Compiler::new(&ast);
        let compile_result = compiler.compile();
        assert!(compile_result.err.is_none());

        let conf = VMConfig {
            trace: false,
            initial_heap_size: 32,
            initial_dreg_count: 64,
        };

        let program = compile_result.program;
        let code_start = compile_result.code_start;
        let jumptab = compile_result.jumptab;
        let fntab = compile_result.fntab;

        let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
        let vm_result = vm.run();
        assert!(vm_result.is_ok());

        vm
    }};
}

#[test]
fn test_stc() {
    let vm = vm_run!("var y = 0;");
    assert_eq!(vm.inspect_heap(String::from("y")), Some(&SifVal::Num(0.0)));
}
