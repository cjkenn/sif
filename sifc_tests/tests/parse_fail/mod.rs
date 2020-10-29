use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use std::fs::File;

const INPUT_PATH: &str = "./tests/parse_fail/inputs";

macro_rules! parse_fail_test {
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            let path = format!("{}/{}.sif", INPUT_PATH, stringify!($test_name));
            let infile = File::open(&path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let result = parser.parse();
            assert_eq!(true, result.has_err);
        }
    };
}

parse_fail_test!(fn_call_wrong_params);
parse_fail_test!(fn_decl_invalid);
parse_fail_test!(fn_decl_wrong_params);
parse_fail_test!(var_decl_invalid);
parse_fail_test!(undecl_sym_assign);
parse_fail_test!(undecl_fn_call);
parse_fail_test!(undecl_fn);
parse_fail_test!(assign_decl);
parse_fail_test!(invalid_param_ident);
parse_fail_test!(fn_decl_unclosed_paren);
parse_fail_test!(fn_decl_eof);
parse_fail_test!(fn_param_count_exceeded);
parse_fail_test!(fn_decl_no_ident);
parse_fail_test!(var_decl_not_ident);
