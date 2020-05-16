#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Eq,
    Lt,
    Gt,
    Period,
    Comma,
    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,

    // Multi character tokens
    EqEq,
    LtEq,
    GtEq,
    EqArrow,
    BangEq,
    AmpAmp,
    PipePipe,

    // Identifiers/literals
    Ident(String),
    Str(String),
    Val(f64),

    // Reserved word literals
    Let,
    Fn,
    Return,
    Record,
    Table,
    Array,
    If,
    Elif,
    Else,
    For,
    In,
    True,
    False,

    // Special type indicating unexpected EOF during lexing/parsing
    Eof,
}

impl TokenType {
    /// True if the TokenType is a binary operator.
    pub fn is_bin_op(&self) -> bool {
        match self {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Percent
            | TokenType::EqEq
            | TokenType::BangEq
            | TokenType::Gt
            | TokenType::Lt
            | TokenType::GtEq
            | TokenType::LtEq => true,
            _ => false,
        }
    }

    /// True if the TokenType is an operator that is expected to work on
    /// numerical values.
    pub fn is_numerical_op(&self) -> bool {
        match self {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Percent => true,
            _ => false,
        }
    }

    /// True if the TokenType is a numerical comparison operator.
    pub fn is_comp_op(&self) -> bool {
        match self {
            TokenType::EqEq
            | TokenType::BangEq
            | TokenType::Gt
            | TokenType::Lt
            | TokenType::GtEq
            | TokenType::LtEq => true,
            _ => false,
        }
    }

    /// True if the TokenType is a logical operator.
    pub fn is_logical_op(&self) -> bool {
        match self {
            TokenType::AmpAmp | TokenType::PipePipe => true,
            _ => false,
        }
    }

    pub fn is_unary_op(&self) -> bool {
        match self {
            TokenType::Minus | TokenType::Bang => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
    pub pos: usize,
}

impl Token {
    pub fn new(ty: TokenType, line: usize, pos: usize) -> Token {
        Token {
            ty: ty,
            line: line,
            pos: pos,
        }
    }

    pub fn get_name(&self) -> String {
        match self.ty {
            TokenType::Ident(ref name) => name.to_string(),
            TokenType::Str(ref name) => name.to_string(),
            _ => panic!("{:?} Wrong token type!", self),
        }
    }

    pub fn get_val(&self) -> f64 {
        match self.ty {
            TokenType::Val(v) => v,
            _ => panic!("{:?} Wrong token type!", self),
        }
    }

    pub fn is_ident(&self) -> bool {
        match self.ty {
            TokenType::Ident(_) => true,
            _ => false,
        }
    }
}
