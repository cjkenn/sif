use crate::token::TokenType;
use std::collections::HashMap;

pub fn get_reserved_words() -> HashMap<String, TokenType> {
    [
        (String::from("let"), TokenType::Let),
        (String::from("fn"), TokenType::Fn),
        (String::from("return"), TokenType::Return),
        (String::from("record"), TokenType::Record),
        (String::from("table"), TokenType::Table),
        (String::from("array"), TokenType::Array),
        (String::from("if"), TokenType::If),
        (String::from("elif"), TokenType::Elif),
        (String::from("else"), TokenType::Else),
        (String::from("for"), TokenType::For),
        (String::from("in"), TokenType::In),
        (String::from("true"), TokenType::True),
        (String::from("false"), TokenType::False),
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn is_reserved_word(word: &str) -> bool {
    let words = get_reserved_words();
    words.contains_key(word)
}
