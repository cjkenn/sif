use val::SifVal;

use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Add {
        line: usize,
        lhs: Box<OpCode>,
        rhs: Box<OpCode>,
    },
    Sub {
        line: usize,
        lhs: Box<OpCode>,
        rhs: Box<OpCode>,
    },
    Mul {
        line: usize,
        lhs: Box<OpCode>,
        rhs: Box<OpCode>,
    },
    Div {
        line: usize,
        lhs: Box<OpCode>,
        rhs: Box<OpCode>,
    },
    Mod {
        line: usize,
        lhs: Box<OpCode>,
        rhs: Box<OpCode>,
    },
    Ret {
        line: usize,
        rhs: Option<Box<OpCode>>,
    },
    Const {
        val: Box<SifVal>,
    },
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty = match self {
            OpCode::Add { .. } => "add".to_string(),
            OpCode::Sub { .. } => "sub".to_string(),
            OpCode::Mul { .. } => "mul".to_string(),
            OpCode::Div { .. } => "div".to_string(),
            OpCode::Mod { .. } => "mod".to_string(),
            OpCode::Ret { .. } => "return".to_string(),
            OpCode::Const { val } => format!("const: {}", &name),
            _ => "unknown op".to_string(),
        };

        write!(f, "{}", pretty)
    }
}
