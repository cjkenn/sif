use sifc_parse::{lex::Lexer, token::TokenTy};
use std::fs::File;

#[test]
fn test_lex_single_char() {
    let path = "./tests/lex_input/single_char";

    let infile = File::open(&path).unwrap();
    let mut lexer = Lexer::new(infile);

    let mut nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::LeftParen);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::RightParen);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::LeftBrace);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::RightBrace);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::LeftBracket);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::RightBracket);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Semicolon);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Period);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Comma);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Plus);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Star);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Percent);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Minus);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::At);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Slash);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Eq);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Lt);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Gt);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Bang);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Amp);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Pipe);
}

#[test]
fn test_lex_multi_char() {
    let path = "./tests/lex_input/multi_char";

    let infile = File::open(&path).unwrap();
    let mut lexer = Lexer::new(infile);

    let mut nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::DoubleLeftBracket);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::DoubleRightBracket);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::EqEq);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::EqArrow);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::LtEq);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::GtEq);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::AmpAmp);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::PipePipe);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Val(10.0));

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Ident("ident".to_string()));

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Str("string".to_string()));
}

#[test]
fn test_lex_reserved_words() {
    let path = "./tests/lex_input/reserved_words";

    let infile = File::open(&path).unwrap();
    let mut lexer = Lexer::new(infile);

    let mut nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Var);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Fn);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Return);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Table);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Array);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::If);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Elif);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::Else);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::For);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::In);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::True);

    nexttkn = lexer.lex();
    assert_eq!(nexttkn.ty, TokenTy::False);
}
