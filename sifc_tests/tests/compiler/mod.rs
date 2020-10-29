use sifc_compiler::{compiler::Compiler, instr};
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use std::fs::File;

const INPUT_PATH: &str = "./tests/compiler/inputs";

macro_rules! compile_test {
    ($test_name:ident, $expected:expr) => {
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
            let mut output = String::from("\n");
            output.push_str(&instr::prog_to_string(compile_result.program));
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

//compile_test! {}
