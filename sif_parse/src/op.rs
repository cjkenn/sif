use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Ret,
    Const { name: String },
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty = match self {
            OpCode::Add => "add".to_string(),
            OpCode::Sub => "sub".to_string(),
            OpCode::Mul => "mul".to_string(),
            OpCode::Div => "div".to_string(),
            OpCode::Mod => "mod".to_string(),
            OpCode::Ret => "return".to_string(),
            OpCode::Const { name } => format!("const: {}", &name),
            _ => "unknown op".to_string(),
        };

        write!(f, "{}", pretty)
    }
}
