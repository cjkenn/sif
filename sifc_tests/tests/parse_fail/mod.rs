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
parse_fail_test!(var_decl_invalid, "var_decl_invalid");
parse_fail_test!(undecl_sym_assign, "undecl_sym_assign");
parse_fail_test!(undecl_fn_call, "undecl_fn_call");
parse_fail_test!(undecl_fn, "undecl_fn");
parse_fail_test!(assign_decl, "assign_decl");
parse_fail_test!(invalid_param_ident, "invalid_param_ident");
parse_fail_test!(fn_decl_unclosed_paren, "fn_decl_unclosed_paren");
parse_fail_test!(fn_decl_eof, "fn_decl_eof");
parse_fail_test!(fn_param_count_exceeded, "fn_param_count_exceeded");
parse_fail_test!(fn_decl_no_ident, "fn_decl_no_ident");
parse_fail_test!(var_decl_not_ident, "var_decl_not_ident");

