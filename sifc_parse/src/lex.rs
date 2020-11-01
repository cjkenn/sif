use crate::{
    reserved::{get_reserved_words, is_reserved_word},
    token::{Token, TokenTy},
};
use sifc_err::{
    err::SifErr,
    lex_err::{LexErr, LexErrTy},
};
use std::io::Read;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
pub struct Lexer<T>
where
    T: Read,
{
    /// Current character in input buffer
    pub curr: Option<char>,

    /// Current line number
    pub line_num: usize,

    /// Current char position in line
    pub line_pos: usize,

    /// Buffer over the input. The input of type T must implement read and seek, which is
    /// mostly intended to handle files or strings (converted)
    reader: BufReader<T>,

    /// Buffer holding the current line
    buffer: Vec<char>,

    /// Reserved words mapping
    reserved: HashMap<String, TokenTy>,

    /// Number of cumulative bytes read by the reader in this lexer
    bytes_read: usize,
}

impl<T> Lexer<T>
where
    T: Read,
{
    pub fn new(infile: T) -> Lexer<T> {
        let mut reader = BufReader::new(infile);
        let mut buf = String::new();
        let init_bytes = reader
            .read_line(&mut buf)
            .expect("reading from lex buffer won't fail");

        let buffer: Vec<char> = buf.chars().collect();
        let curr_ch = if buffer.len() == 0 {
            None
        } else {
            Some(buffer[0])
        };

        Lexer {
            curr: curr_ch,
            line_num: 1,
            line_pos: 0,
            reader: reader,
            buffer: buffer,
            reserved: get_reserved_words(),
            bytes_read: init_bytes,
        }
    }

    /// Get the next token from the input stream. If this returns None, it means we're either
    /// at the end of the input, or we've encountered a character we don't recognize.
    pub fn lex(&mut self) -> Token {
        if self.curr.is_none() {
            return self.eof_tkn();
        }

        // Skip whitespace
        self.skip_whitespace();
        if self.curr.is_none() {
            return self.eof_tkn();
        }

        while self.curr.unwrap() == '#' {
            self.advance_to_next_line();
            if self.curr.is_none() {
                return self.eof_tkn();
            }
        }

        let ch = self.curr.unwrap();
        match ch {
            '(' => self.consume(TokenTy::LeftParen),
            ')' => self.consume(TokenTy::RightParen),
            '{' => self.consume(TokenTy::LeftBrace),
            '}' => self.consume(TokenTy::RightBrace),
            '[' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '[' => {
                        let tkn = self.consume(TokenTy::DoubleLeftBracket);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::LeftBracket),
                }
            }
            ']' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == ']' => {
                        let tkn = self.consume(TokenTy::DoubleRightBracket);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::RightBracket),
                }
            }
            ';' => self.consume(TokenTy::Semicolon),
            '.' => self.consume(TokenTy::Period),
            ',' => self.consume(TokenTy::Comma),
            '+' => self.consume(TokenTy::Plus),
            '*' => self.consume(TokenTy::Star),
            '%' => self.consume(TokenTy::Percent),
            '"' => self.lex_str(),
            '-' => self.consume(TokenTy::Minus),
            '@' => self.consume(TokenTy::At),
            '/' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '/' => {
                        while self.curr.unwrap() != '\n' {
                            self.advance();
                        }
                        return self.lex();
                    }
                    _ => self.consume(TokenTy::Slash),
                }
            }
            '=' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '=' => {
                        let tkn = self.consume(TokenTy::EqEq);
                        self.advance();
                        tkn
                    }
                    Some(ch) if ch == '>' => {
                        let tkn = self.consume(TokenTy::EqArrow);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Eq),
                }
            }
            '<' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '=' => {
                        let tkn = self.consume(TokenTy::LtEq);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Lt),
                }
            }
            '>' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '=' => {
                        let tkn = self.consume(TokenTy::GtEq);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Gt),
                }
            }
            '!' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '=' => {
                        let tkn = self.consume(TokenTy::BangEq);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Bang),
                }
            }
            '&' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '&' => {
                        let tkn = self.consume(TokenTy::AmpAmp);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Amp),
                }
            }
            '|' => {
                let nextch = self.peek_char();
                match nextch {
                    Some(ch) if ch == '|' => {
                        let tkn = self.consume(TokenTy::PipePipe);
                        self.advance();
                        tkn
                    }
                    _ => self.consume(TokenTy::Pipe),
                }
            }
            _ if ch.is_digit(10) => self.lex_num(),
            _ if ch.is_alphabetic() => self.lex_ident(),
            _ => {
                LexErr::new(self.line_num, self.line_pos, LexErrTy::UnknownChar(ch)).emit();
                self.eof_tkn()
            }
        }
    }

    /// Lex a string literal. We expect to have a " character when this
    /// function is called, and we consume the last " character during
    /// this call.
    fn lex_str(&mut self) -> Token {
        let mut lit = String::new();
        let startpos = self.line_pos;
        let startline = self.line_num;

        // Consume '"'
        self.advance();

        while !self.finished() {
            match self.curr {
                Some(ch) => {
                    if ch == '"' {
                        return self.consume_w_pos(TokenTy::Str(lit), startline, startpos);
                    } else {
                        lit.push(ch);
                        self.advance();
                    }
                }
                None => {
                    LexErr::new(
                        self.line_num,
                        self.line_pos,
                        LexErrTy::UnterminatedString(lit),
                    )
                    .emit();
                    return self.eof_tkn();
                }
            }
        }

        // If we finished lexing here without returning, the file
        // is fully lexed without a string termination occurring.
        LexErr::new(
            self.line_num,
            self.line_pos,
            LexErrTy::UnterminatedString(lit),
        )
        .emit();
        self.eof_tkn()
    }

    /// Lex a floating point or integer literal.
    fn lex_num(&mut self) -> Token {
        let mut lit = String::new();
        let startpos = self.line_pos;
        let startline = self.line_num;

        let mut currch = self.curr;

        while let Some(ch) = currch {
            if ch.is_digit(10) {
                lit.push(ch);
                self.advance();
                currch = self.curr;
            } else if ch == '.' {
                lit.push(ch);
                self.advance();
                let mut innerch = self.curr;

                while let Some(ch) = innerch {
                    if ch.is_digit(10) {
                        lit.push(ch);
                        self.advance();
                        innerch = self.curr;
                    } else {
                        innerch = None;
                        currch = None;
                    }
                }
            } else {
                currch = None;
            }
        }

        let numval = lit.parse::<f64>().unwrap();
        Token::new(TokenTy::Val(numval), startline, startpos)
    }

    /// Lex an identifier. This is not a string literal and does not
    /// contain quotations around it.
    fn lex_ident(&mut self) -> Token {
        let mut lit = String::new();
        let startpos = self.line_pos;
        let startline = self.line_num;

        let mut currch = self.curr;

        while let Some(ch) = currch {
            if ch.is_alphanumeric() {
                lit.push(ch);
                self.advance();
                currch = self.curr;
            } else {
                currch = None;
            }
        }

        let mut ty = TokenTy::Ident(lit.clone());

        if is_reserved_word(&lit) {
            ty = self.reserved.get(&lit).unwrap().clone();
        }

        Token::new(ty, startline, startpos)
    }

    /// Consume current char and return a token from it.
    fn consume(&mut self, ty: TokenTy) -> Token {
        let tkn = Token::new(ty, self.line_num, self.line_pos);
        self.advance();
        tkn
    }

    /// Consume current char and return a token from it, given a line
    /// and char position. Used so that the correct line/pos combo can be reported
    /// for identifiers, literals, and numbers.
    fn consume_w_pos(&mut self, ty: TokenTy, line: usize, line_pos: usize) -> Token {
        let tkn = Token::new(ty, line, line_pos);
        self.advance();
        tkn
    }

    /// Return the next char in the buffer, if any.
    fn peek_char(&mut self) -> Option<char> {
        if self.line_pos >= self.buffer.len() - 1 {
            return None;
        }

        Some(self.buffer[self.line_pos + 1])
    }

    /// Move the char position ahead by 1. If we are at the end of the current
    /// buffer, reads the next line of the file into the buffer and sets
    /// the position to 0.
    fn advance(&mut self) {
        let on_new_line = match self.curr {
            Some(ch) if ch == '\n' => true,
            _ => false,
        };

        if self.line_pos == self.buffer.len() - 1 || on_new_line {
            self.next_line();
        } else {
            self.line_pos = self.line_pos + 1;
        }

        if self.finished() {
            self.curr = None;
        } else {
            self.curr = Some(self.buffer[self.line_pos]);
        }
    }

    fn advance_to_next_line(&mut self) {
        while self.curr.unwrap() != '\n' {
            self.advance();
            if self.curr.is_none() {
                return;
            }
        }

        // We are on a \n character here, so move ahead to next line.
        self.advance();
        self.skip_whitespace();
    }

    fn skip_whitespace(&mut self) {
        while self.curr.is_some() && self.curr.unwrap().is_whitespace() {
            self.advance();
        }
    }

    /// Read the next line of the input file into the buffer.
    fn next_line(&mut self) {
        let mut buf = String::new();
        let line_bytes = self
            .reader
            .read_line(&mut buf)
            .expect("file reader should not fail");
        self.buffer = buf.chars().collect();
        self.bytes_read = self.bytes_read + line_bytes;

        self.line_pos = 0;
        self.line_num = self.line_num + 1;
    }

    /// When the input buffer is empty, that means read_line has indicated
    /// we're at the end of the file.
    fn finished(&self) -> bool {
        self.buffer.len() == 0
    }

    fn eof_tkn(&self) -> Token {
        Token::new(TokenTy::Eof, self.line_num, self.line_pos)
    }
}
