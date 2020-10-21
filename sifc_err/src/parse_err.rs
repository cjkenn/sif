use crate::err::SifErr;

#[derive(Debug, Clone)]
pub enum ParseErrTy {
    InvalidIdent(String),
    InvalidTkn(String),
    InvalidAssign(String),
    InvalidForStmt,
    InvalidIfStmt,
    TknMismatch(String, String),
    FnParmCntExceeded(usize),
    WrongFnParmCnt(usize, usize),
    UndeclSym(String),
    UnassignedVar(String),
    ExpectedIdent(String),
}

#[derive(Debug, Clone)]
pub struct ParseErr {
    pub line: usize,
    pub pos: usize,
    pub ty: ParseErrTy,
}

impl ParseErr {
    pub fn new(line: usize, pos: usize, ty: ParseErrTy) -> ParseErr {
        ParseErr {
            line: line,
            pos: pos,
            ty: ty,
        }
    }

    pub fn continuable(self) -> bool {
        match self.ty {
            ParseErrTy::TknMismatch(_, _) => false,
            ParseErrTy::FnParmCntExceeded(_) => false,
            _ => true,
        }
    }
}

impl SifErr for ParseErr {
    fn emit(&self) {
        eprintln!("sif: Parse error - {}", self.to_msg());
    }

    fn to_msg(&self) -> String {
        let str_pos = format!("[Line {}:{}]", self.line, self.pos);

        match self.ty {
            ParseErrTy::InvalidIdent(ref found) => {
                format!("{} Invalid identifier '{}' found", str_pos, found)
            }
            ParseErrTy::InvalidTkn(ref found) => {
                format!("{} Invalid token '{}' found", str_pos, found)
            }
            ParseErrTy::InvalidAssign(ref found) => {
                format!("{} '{}' is not a valid assignment value", str_pos, found)
            }
            ParseErrTy::InvalidForStmt => format!(
                "{} Invalid for loop: must start with a variable declaration",
                str_pos
            ),
            ParseErrTy::InvalidIfStmt => format!(
                "{} Invalid if statement: cannot contain more than one else condition",
                str_pos
            ),
            ParseErrTy::TknMismatch(ref expected, ref found) => format!(
                "{} Expected token '{}', but found '{}'",
                str_pos, expected, found
            ),
            ParseErrTy::FnParmCntExceeded(ref expected) => {
                format!("{} Parameter count exceeds limit of {}", str_pos, expected)
            }
            ParseErrTy::WrongFnParmCnt(ref expected, ref found) => format!(
                "{} Expected {} parameters, but found {}",
                str_pos, expected, found
            ),
            ParseErrTy::UnassignedVar(ref found) => format!(
                "{} Cannot reference un-assigned variable '{}'",
                str_pos, found
            ),
            ParseErrTy::UndeclSym(ref found) => {
                format!("{} Undeclared symbol '{}' found", str_pos, found)
            }
            ParseErrTy::ExpectedIdent(ref found) => format!(
                "{} Identifier expected, found '{}'. Is this a reserved word?",
                str_pos, found
            ),
        }
    }
}
