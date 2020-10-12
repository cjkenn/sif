use crate::token::TokenTy;

use std::collections::HashMap;

pub fn get_reserved_words() -> HashMap<String, TokenTy> {
    [
        (String::from("let"), TokenTy::Let),
        (String::from("fn"), TokenTy::Fn),
        (String::from("return"), TokenTy::Return),
        (String::from("record"), TokenTy::Record),
        (String::from("table"), TokenTy::Table),
        (String::from("array"), TokenTy::Array),
        (String::from("if"), TokenTy::If),
        (String::from("elif"), TokenTy::Elif),
        (String::from("else"), TokenTy::Else),
        (String::from("for"), TokenTy::For),
        (String::from("in"), TokenTy::In),
        (String::from("true"), TokenTy::True),
        (String::from("false"), TokenTy::False),
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn is_reserved_word(word: &str) -> bool {
    let words = get_reserved_words();
    words.contains_key(word)
}
