use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum LexErrTy {
    UnknownChar(char),
    UnterminatedString(String),
}

pub struct LexErr {
    pub line: usize,
    pub pos: usize,
    pub ty: LexErrTy,
}

impl LexErr {
    pub fn new(line: usize, pos: usize, ty: LexErrTy) -> LexErr {
        LexErr {
            line: line,
            pos: pos,
            ty: ty,
        }
    }
}

impl SifErr for LexErr {
    fn emit(&self) {
        eprintln!("sif: Parse error - {}", self.to_msg());
    }

    fn to_msg(&self) -> String {
        let str_pos = format!("[Line {}:{}]", self.line, self.pos);

        match self.ty {
            LexErrTy::UnknownChar(ref ch) => format!("{} Unrecognized character '{}'", str_pos, ch),
            LexErrTy::UnterminatedString(ref found) => {
                format!("{} Unterminated string literal '{}'", str_pos, found)
            }
        }
    }
}
