use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use std::fs::File;

const INPUT_PATH: &str = "./tests/parse_pass/inputs";

macro_rules! parse_pass_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            let path = format!("{}/{}.sif", INPUT_PATH, stringify!($test_name));
            let infile = File::open(&path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let result = parser.parse();
            assert_eq!(false, result.has_err);
        }
    };
}

parse_pass_test!(if_stmt);
parse_pass_test!(for_stmt);
parse_pass_test!(var_decl);
parse_pass_test!(fn_decl);
parse_pass_test!(fn_w_ret_stmt);
parse_pass_test!(fn_call);
parse_pass_test!(exprs);
parse_pass_test!(table_decl);
parse_pass_test!(array_decl);
parse_pass_test!(array_mut_assign);
