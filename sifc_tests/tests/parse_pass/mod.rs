use sifc_parse::{
    lex::Lexer,
    parser::Parser,
    symtab::SymTab,
};
use std::fs::File;

macro_rules! parse_pass_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            let path = format!("./tests/parse_pass/inputs/{}.sif", $file_name);
            let infile = File::open(&path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let result = parser.parse();
            assert_eq!(false, result.has_err);
        }
    };
}

parse_pass_test!(if_stmt, "if_stmt");
parse_pass_test!(for_stmt, "for_stmt");
parse_pass_test!(var_decl, "var_decl");
parse_pass_test!(fn_decl, "fn_decl");
parse_pass_test!(fn_w_ret_stmt, "fn_w_ret_stmt");
parse_pass_test!(fn_call, "fn_call");
parse_pass_test!(exprs, "exprs");
parse_pass_test!(table_decl, "table_decl");
parse_pass_test!(array_decl, "array_decl");
