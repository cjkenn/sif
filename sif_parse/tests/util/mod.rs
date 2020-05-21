use sif_parse::parser::ParserResult;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub struct ParseTestCtx {
    pub path: String,
    pub is_pass: bool,
    pub line: usize,
    pub pos: usize,
}

pub fn setup(test_name: &str) -> ParseTestCtx {
    let path = get_test_path(test_name);
    let expfile = File::open(path).unwrap();
    let exp = BufReader::new(expfile).lines().next().unwrap().unwrap();

    let pe = match parse_expectations(&exp, path) {
        Ok(p) => p,
        Err(e) => panic!(e),
    };

    pe
}

pub fn check(test_name: &str, pctx: ParseTestCtx, result: ParserResult) {
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

/// Returns a listing of test names to their file locations.
fn get_test_path(test_name: &str) -> &str {
    let paths: HashMap<&str, &str> = [
        ("if_stmt", "./tests/input/if_stmt.sif"),
        ("for_stmt", "./tests/input/for_stmt.sif"),
        ("var_decl", "./tests/input/var_decl.sif"),
        ("var_decl_invalid", "./tests/input/var_decl_invalid.sif"),
        ("fn_decl_valid", "./tests/input/fn_decl_valid.sif"),
        ("fn_decl_invalid", "./tests/input/fn_decl_invalid.sif"),
        ("fn_w_ret_stmt", "./tests/input/fn_w_ret_stmt.sif"),
        ("fn_call", "./tests/input/fn_call.sif"),
        ("exprs", "./tests/input/exprs.sif"),
        ("table_decl", "./tests/input/table_decl.sif"),
        ("record_decl", "./tests/input/record_decl.sif"),
        ("array_decl", "./tests/input/array_decl.sif"),
    ]
    .iter()
    .cloned()
    .collect();

    let path = paths.get(test_name);
    match path {
        Some(p) => return p,
        None => panic!("invalid parser test name provided!"),
    };
}

fn parse_expectations(expectations: &str, test_path: &str) -> Result<ParseTestCtx, &'static str> {
    let parts: Vec<&str> = expectations.split("::").collect();

    if parts.len() == 0 {
        return Err("No test expectations string found");
    }

    if parts.len() < 2 {
        return Err(
            "Invalid test expectation string. Usage: 'expect::[pass][fail]::[line]::[pos]'",
        );
    }

    if !parts[0].contains("expect") {
        return Err(
            "Invalid test expectation string. Usage: 'expect::[pass][fail]::[line]::[pos]'",
        );
    }

    if parts[1] != "fail" && parts[1] != "pass" {
        return Err(
            "Invalid test expectation string. Usage: 'expect::[pass][fail]::[line]::[pos]'",
        );
    }

    if parts[1] == "pass" {
        return Ok(ParseTestCtx {
            path: String::from(test_path),
            is_pass: true,
            line: 0,
            pos: 0,
        });
    }

    if parts.len() < 4 {
        return Err(
            "Invalid test expectation string. Usage: 'expect::[pass][fail]::[line]::[pos]'",
        );
    }

    let line = parts[2].parse::<usize>();
    if line.is_err() {
        return Err("Line number in expectations must be valid int");
    }

    let pos = parts[3].parse::<usize>();
    if pos.is_err() {
        return Err("Position number in expectations must be valid int");
    }

    Ok(ParseTestCtx {
        path: String::from(test_path),
        is_pass: false,
        line: line.unwrap(),
        pos: pos.unwrap(),
    })
}
