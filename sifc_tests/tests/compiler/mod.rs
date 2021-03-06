use sifc_bytecode::{compiler::Compiler, instr::Instr};
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use std::fs::File;

const INPUT_PATH: &str = "./tests/compiler/inputs";

/// Converts a program into a string. This uses the fmt::Display
/// implementation for Instr and does not contain section names.
/// It always starts and ends with a newline. Line information for each instruction
/// is excluded. If that is needed, use fmt::Debug.
fn prog_to_string(prog: Vec<Instr>) -> String {
    let mut result = String::from("\n");
    for i in prog {
        result.push_str(&format!("{:#}\n", i));
    }
    result
}

// Expects a test/file name as first arg, and then the expected output as a second arg.
macro_rules! compile_test {
    ($test_name:ident, $expected:expr) => {
        #[test]
        fn $test_name() {
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

            let output = prog_to_string(compile_result.program);
            assert_eq!(output, $expected);
        }
    };
}

compile_test! {
    single_var,
    r"
lbl0: stc 0 g
"
}

compile_test! {
    multi_var,
    r"
lbl0: stc 0 g
lbl0: stc hello t
lbl0: stc false r
"
}

compile_test! {
    bin_op,
    r"
lbl0: ldc 10 r0
lbl0: ldc 10 r1
lbl0: add r0 r1 r2
lbl0: str r2 g
"
}

compile_test! {
    nneg_op,
    r"
lbl0: ldc 10 r0
lbl0: nneg r0 r1
lbl0: str r1 g
"
}

compile_test! {
    lneg_op,
    r"
lbl0: ldc false r0
lbl0: lneg r0 r1
lbl0: str r1 g
"
}

compile_test! {
    empty_fn,
    r"
lbl0: fn @x []
lbl0: ret
"
}

compile_test! {
    fn_w_params,
    r#"
lbl0: fn @x ["y"]
lbl0: fstpop r0
lbl0: str r0 y
lbl0: ldn y r1
lbl0: fstpush r1
lbl0: ret
"#
}

compile_test! {
    fn_call,
    r#"
lbl0: fn @x ["y"]
lbl0: fstpop r0
lbl0: str r0 y
lbl0: ldn y r1
lbl0: fstpush r1
lbl0: ret
lbl1: ldc 1 r2
lbl1: fstpush r2
lbl1: call x
lbl1: fstpop r3
lbl1: str r3 g
"#
}

compile_test! {
    for_stmt,
    r"
lbl0: stc [Num(1.0), Num(2.0), Num(3.0)] g
lbl0: stc 0 x
lbl0: stc 0 idx
lbl0: ldas g r1
lbl1: ldn idx r0
lbl1: ldav g r0 r2
lbl1: str r2 val
lbl1: ldn x r3
lbl1: ldn val r4
lbl1: add r3 r4 r5
lbl1: str r5 x
lbl1: incrr r0
lbl1: str r0 idx
lbl1: lt r0 r1 r6
lbl1: jmpt r6 lbl1
"
}

compile_test! {
    std_lib_call,
    r"
lbl0: ldc hello world r0
lbl0: fstpush r0
lbl0: stdcall print
lbl0: fstpop r1
"
}

compile_test! {
    basic_if,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl2
lbl2: nop
"
}

compile_test! {
    basic_if_else,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl3
lbl2: stc 1 t
lbl3: nop
"
}

compile_test! {
    if_elif,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl6
lbl2: ldc true r3
lbl2: ldc true r4
lbl2: and r3 r4 r5
lbl2: jmpf r5 lbl4
lbl3: stc 1 t
lbl3: jmpa lbl6
lbl4: ldc false r6
lbl4: ldc false r7
lbl4: or r6 r7 r8
lbl4: jmpf r8 lbl6
lbl5: stc 2 t
lbl5: jmpa lbl6
lbl6: nop
"
}

compile_test! {
    if_elif_else,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl7
lbl2: ldc true r3
lbl2: ldc true r4
lbl2: and r3 r4 r5
lbl2: jmpf r5 lbl4
lbl3: stc 1 t
lbl3: jmpa lbl7
lbl4: ldc false r6
lbl4: ldc false r7
lbl4: or r6 r7 r8
lbl4: jmpf r8 lbl6
lbl5: stc 2 t
lbl5: jmpa lbl7
lbl6: stc 3 t
lbl7: nop
"
}

compile_test! {
    nested_if,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 1 t
lbl1: jmpa lbl6
lbl2: ldc false r3
lbl2: ldc false r4
lbl2: and r3 r4 r5
lbl2: jmpf r5 lbl4
lbl3: stc 0 t
lbl3: jmpa lbl5
lbl4: stc 2 t
lbl5: nop
lbl6: nop
"
}

compile_test! {
    complex_if,
    r"
lbl0: ldc true r0
lbl0: ldc false r1
lbl0: or r0 r1 r2
lbl0: jmpf r2 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl8
lbl2: ldc false r3
lbl2: ldc false r4
lbl2: and r3 r4 r5
lbl2: jmpf r5 lbl6
lbl3: ldc true r6
lbl3: ldc true r7
lbl3: and r6 r7 r8
lbl3: jmpf r8 lbl5
lbl4: stc 1 t
lbl4: jmpa lbl5
lbl5: nop
lbl5: stc 2 t
lbl5: jmpa lbl8
lbl6: ldc true r9
lbl6: ldc false r10
lbl6: or r9 r10 r11
lbl6: jmpf r11 lbl8
lbl7: stc 3 t
lbl7: jmpa lbl8
lbl8: nop
"
}

compile_test! {
    fn_decl_if,
    r#"
lbl0: fn @t ["x"]
lbl0: fstpop r0
lbl0: str r0 x
lbl0: ldc true r1
lbl0: ldc false r2
lbl0: or r1 r2 r3
lbl0: jmpf r3 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl2
lbl2: nop
lbl2: ret
"#
}

compile_test! {
    fn_decl_if_else,
    r#"
lbl0: fn @t ["x"]
lbl0: fstpop r0
lbl0: str r0 x
lbl0: ldc true r1
lbl0: ldc false r2
lbl0: or r1 r2 r3
lbl0: jmpf r3 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl3
lbl2: stc 1 t
lbl3: nop
lbl3: ret
"#
}

compile_test! {
    fn_decl_if_elif,
    r#"
lbl0: fn @t ["x"]
lbl0: fstpop r0
lbl0: str r0 x
lbl0: ldc true r1
lbl0: ldc false r2
lbl0: or r1 r2 r3
lbl0: jmpf r3 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl4
lbl2: ldc false r4
lbl2: ldc false r5
lbl2: and r4 r5 r6
lbl2: jmpf r6 lbl4
lbl3: stc 1 t
lbl3: jmpa lbl4
lbl4: nop
lbl4: ret
"#
}

compile_test! {
    fn_decl_if_elif_else,
    r#"
lbl0: fn @t ["x"]
lbl0: fstpop r0
lbl0: str r0 x
lbl0: ldc true r1
lbl0: ldc false r2
lbl0: or r1 r2 r3
lbl0: jmpf r3 lbl2
lbl1: stc 0 t
lbl1: jmpa lbl5
lbl2: ldc false r4
lbl2: ldc false r5
lbl2: and r4 r5 r6
lbl2: jmpf r6 lbl4
lbl3: stc 1 t
lbl3: jmpa lbl5
lbl4: stc 2 t
lbl5: nop
lbl5: ret
"#
}

compile_test! {
    fn_decl_for_stmt,
    r#"
lbl0: fn @t ["x"]
lbl0: fstpop r0
lbl0: str r0 x
lbl0: stc [Num(1.0), Num(2.0), Num(3.0)] arr
lbl0: stc 0 i
lbl0: ldas arr r2
lbl1: ldn i r1
lbl1: ldav arr r1 r3
lbl1: str r3 v
lbl1: ldn v r4
lbl1: fstpush r4
lbl1: stdcall print
lbl1: fstpop r5
lbl1: incrr r1
lbl1: str r1 i
lbl1: lt r1 r2 r6
lbl1: jmpt r6 lbl1
lbl1: ret
"#
}

compile_test! {
    array_mut,
    r"
lbl0: stc [Num(1.0), Num(2.0)] g
lbl0: ldc 3 r0
lbl0: ldc 1 r1
lbl0: upda g r0 r1
"
}

compile_test! {
    for_stmt_fn_call,
    r"
lbl0: ldc 0 r1
lbl0: fstpush r1
lbl0: ldc 1 r2
lbl0: fstpush r2
lbl0: stdcall range
lbl0: fstpop r3
lbl0: str r3 fortmp
lbl0: stc 0 i
lbl0: ldas fortmp r4
lbl1: ldn i r0
lbl1: ldav fortmp r0 r5
lbl1: str r5 v
lbl1: stc 0 y
lbl1: incrr r0
lbl1: str r0 i
lbl1: lt r0 r4 r6
lbl1: jmpt r6 lbl1
"
}
