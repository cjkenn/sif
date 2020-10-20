use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenTy {
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
    Arrow,
    EqArrow,
    BangEq,
    AmpAmp,
    PipePipe,
    DoubleLeftBracket,
    DoubleRightBracket,

    // Identifiers/literals
    Ident(String),
    Str(String),
    Val(f64),

    // Reserved word literals
    Var,
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

impl TokenTy {
    /// True if the TokenTy is a binary operator.
    pub fn is_bin_op(&self) -> bool {
        match self {
            TokenTy::Plus
            | TokenTy::Minus
            | TokenTy::Star
            | TokenTy::Slash
            | TokenTy::Percent
            | TokenTy::EqEq
            | TokenTy::BangEq
            | TokenTy::Gt
            | TokenTy::Lt
            | TokenTy::GtEq
            | TokenTy::LtEq => true,
            _ => false,
        }
    }

    /// True if the TokenTy is an operator that is expected to work on
    /// numerical values.
    pub fn is_numerical_op(&self) -> bool {
        match self {
            TokenTy::Plus | TokenTy::Minus | TokenTy::Star | TokenTy::Slash | TokenTy::Percent => {
                true
            }
            _ => false,
        }
    }

    /// True if the TokenTy is a numerical comparison operator.
    pub fn is_comp_op(&self) -> bool {
        match self {
            TokenTy::EqEq
            | TokenTy::BangEq
            | TokenTy::Gt
            | TokenTy::Lt
            | TokenTy::GtEq
            | TokenTy::LtEq => true,
            _ => false,
        }
    }

    /// True if the TokenTy is a logical operator.
    pub fn is_logical_op(&self) -> bool {
        match self {
            TokenTy::AmpAmp | TokenTy::PipePipe => true,
            _ => false,
        }
    }

    pub fn is_unary_op(&self) -> bool {
        match self {
            TokenTy::Minus | TokenTy::Bang => true,
            _ => false,
        }
    }
}

impl fmt::Display for TokenTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty_ty = match self {
            TokenTy::LeftParen => "(".to_string(),
            TokenTy::RightParen => ")".to_string(),
            TokenTy::LeftBrace => "{".to_string(),
            TokenTy::RightBrace => "}".to_string(),
            TokenTy::LeftBracket => "[".to_string(),
            TokenTy::RightBracket => "]".to_string(),
            TokenTy::Semicolon => ";".to_string(),
            TokenTy::Eq => "=".to_string(),
            TokenTy::Lt => "<".to_string(),
            TokenTy::Gt => ">".to_string(),
            TokenTy::Period => ".".to_string(),
            TokenTy::Comma => ",".to_string(),
            TokenTy::Bang => "!".to_string(),
            TokenTy::Plus => "+".to_string(),
            TokenTy::Minus => "-".to_string(),
            TokenTy::Star => "*".to_string(),
            TokenTy::Slash => "/".to_string(),
            TokenTy::Percent => "%".to_string(),
            TokenTy::Amp => "&".to_string(),
            TokenTy::Pipe => "|".to_string(),

            TokenTy::EqEq => "==".to_string(),
            TokenTy::LtEq => "<=".to_string(),
            TokenTy::GtEq => ">=".to_string(),
            TokenTy::BangEq => "!=".to_string(),
            TokenTy::AmpAmp => "&&".to_string(),
            TokenTy::PipePipe => "||".to_string(),
            TokenTy::Arrow => "->".to_string(),
            TokenTy::EqArrow => "=>".to_string(),
            TokenTy::DoubleLeftBracket => "[[".to_string(),
            TokenTy::DoubleRightBracket => "]]".to_string(),

            TokenTy::Ident(name) => format!("{}", name),
            TokenTy::Str(name) => format!("{}", name),
            TokenTy::Val(val) => format!("{}", val),

            TokenTy::Var => "var".to_string(),
            TokenTy::Fn => "fn".to_string(),
            TokenTy::Return => "return".to_string(),
            TokenTy::Record => "record".to_string(),
            TokenTy::Table => "table".to_string(),
            TokenTy::Array => "array".to_string(),
            TokenTy::If => "if".to_string(),
            TokenTy::Elif => "elif".to_string(),
            TokenTy::Else => "else".to_string(),
            TokenTy::For => "for".to_string(),
            TokenTy::In => "in".to_string(),
            TokenTy::True => "true".to_string(),
            TokenTy::False => "false".to_string(),

            TokenTy::Eof => "EOF".to_string(),
        };

        write!(f, "{}", pretty_ty)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub ty: TokenTy,
    pub line: usize,
    pub pos: usize,
}

impl Token {
    pub fn new(ty: TokenTy, line: usize, pos: usize) -> Token {
        Token {
            ty: ty,
            line: line,
            pos: pos,
        }
    }

    pub fn get_name(&self) -> String {
        match self.ty {
            TokenTy::Ident(ref name) => name.to_string(),
            TokenTy::Str(ref name) => name.to_string(),
            _ => panic!("{:?} Wrong token type!", self),
        }
    }

    pub fn get_val(&self) -> f64 {
        match self.ty {
            TokenTy::Val(v) => v,
            _ => panic!("{:?} Wrong token type!", self),
        }
    }

    pub fn is_ident(&self) -> bool {
        match self.ty {
            TokenTy::Ident(_) => true,
            _ => false,
        }
    }
}
