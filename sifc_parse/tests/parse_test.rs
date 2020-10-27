use sifc_parse::{
    lex::Lexer,
    parser::{Parser, ParserResult},
    symtab::SymTab,
};
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct ParseTestCtx {
    pub path: String,
    pub is_pass: bool,
}

fn setup(test_name: &str) -> ParseTestCtx {
    let path = format!("./tests/parse_input/{}.sif", test_name);
    let expfile = File::open(&path).unwrap();
    let exp = BufReader::new(expfile).lines().next().unwrap().unwrap();

    let pe = match parse_expectations(&exp, &path) {
        Ok(p) => p,
        Err(e) => panic!(e),
    };

    pe
}

fn parse_expectations(expectations: &str, test_path: &str) -> Result<ParseTestCtx, &'static str> {
    let parts: Vec<&str> = expectations.split("::").collect();

    if parts.len() == 0 {
        return Err("No test expectations string found");
    }

    if parts.len() < 2 {
        return Err("Invalid test expectation string. Usage: 'expect::[pass][fail]'");
    }

    if !parts[0].contains("expect") {
        return Err("Invalid test expectation string. Usage: 'expect::[pass][fail]'");
    }

    if parts[1] != "fail" && parts[1] != "pass" {
        return Err("Invalid test expectation string. Usage: 'expect::[pass][fail]'");
    }

    if parts[1] == "pass" {
        return Ok(ParseTestCtx {
            path: String::from(test_path),
            is_pass: true,
        });
    }

    Ok(ParseTestCtx {
        path: String::from(test_path),
        is_pass: false,
    })
}

fn check(pctx: ParseTestCtx, result: ParserResult) {
    match pctx.is_pass {
        true => {
            if result.has_err {
                assert!(false, "FAIL: {:?} expected successful parse, found error");
            } else {
                assert!(true);
            }
        }
        false => {
            if !result.has_err {
                assert!(false, "FAIL: {:?} expected error, found none",);
            } else {
                assert!(true);
            }
        }
    }
}

macro_rules! parser_test {
    ($test_name:ident, $file_name:expr) => {
        #[test]
        fn $test_name() {
            let pctx = setup($file_name);

            let infile = File::open(&pctx.path).unwrap();
            let mut symtab = SymTab::new();
            let mut lex = Lexer::new(infile);
            let mut parser = Parser::new(&mut lex, &mut symtab);

            let result = parser.parse();
            check(pctx, result);
        }
    };
}

parser_test!(if_stmt, "if_stmt");
parser_test!(for_stmt, "for_stmt");
parser_test!(var_decl, "var_decl");
parser_test!(var_decl_invalid, "var_decl_invalid");
parser_test!(fn_decl_valid, "fn_decl_valid");
parser_test!(fn_decl_invalid, "fn_decl_invalid");
parser_test!(fn_decl_wrong_params, "fn_decl_wrong_params");
parser_test!(fn_w_ret_stmt, "fn_w_ret_stmt");
parser_test!(fn_call, "fn_call");
parser_test!(fn_call_wrong_params, "fn_call_wrong_params");
parser_test!(exprs, "exprs");
parser_test!(table_decl, "table_decl");
parser_test!(array_decl, "array_decl");
parser_test!(table_access_invalid, "table_access_invalid");
