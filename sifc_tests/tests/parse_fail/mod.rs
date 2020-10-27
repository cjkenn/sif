use sifc_parse::{
    lex::Lexer,
    parser::Parser,
    symtab::SymTab,
};
use std::fs::File;

macro_rules! parse_fail_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            let path = format!("./tests/parse_fail/inputs/{}.sif", $file_name);
            let infile = File::open(&path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let result = parser.parse();
            assert_eq!(true, result.has_err);
        }
    };
}

parse_fail_test!(fn_call_wrong_params, "fn_call_wrong_params");
parse_fail_test!(fn_decl_invalid, "fn_decl_invalid");
parse_fail_test!(fn_decl_wrong_params, "fn_decl_wrong_params");
parse_fail_test!(table_access_invalid, "table_access_invalid");
parse_fail_test!(var_decl_invalid, "var_decl_invalid");
