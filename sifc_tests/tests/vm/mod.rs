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
fn stc() {
    let vm = vm_run!("var y = 0;");
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(0.0)));
}

#[test]
fn stn() {
    let vm = vm_run!("var y = 0; var x = y;");
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(0.0)));
    assert_eq!(vm.inspect_heap("x"), Some(&SifVal::Num(0.0)));
}

#[test]
fn ldc() {
    let mut vm = vm_run!("var y = 1 + 2;");
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Num(2.0)));
    assert_eq!(vm.inspect_dreg(2), Some(SifVal::Num(3.0)));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(3.0)));
}

#[test]
fn ldn() {
    let mut vm = vm_run!("var y = 1; var x = 2 + y;");
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Num(2.0)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_dreg(2), Some(SifVal::Num(3.0)));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(1.0)));
    assert_eq!(vm.inspect_heap("x"), Some(&SifVal::Num(3.0)));
}

#[test]
fn nneg() {
    let mut vm = vm_run!("var y = -1;");
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Num(-1.0)));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(-1.0)));
}

#[test]
fn lneg() {
    let mut vm = vm_run!("var y = !true;");
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Bl(true)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Bl(false)));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Bl(false)));
}

#[test]
fn stdcall_range() {
    let mut vm = vm_run!("var y = @range(0,1);");
    let expected_vec = vec![SifVal::Num(0.0), SifVal::Num(1.0)];
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Num(0.0)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_dreg(2), Some(SifVal::Arr(expected_vec.clone())));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Arr(expected_vec)));
}

#[test]
fn fncall() {
    let mut vm = vm_run!("fn x() { return 1; } var y = x();");
    assert_eq!(vm.inspect_dreg(0), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_dreg(1), Some(SifVal::Num(1.0)));
    assert_eq!(vm.inspect_heap("y"), Some(&SifVal::Num(1.0)));
}
