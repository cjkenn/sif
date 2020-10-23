use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum SifVal {
    Num(f64),
    Str(String),
    Bl(bool),
    Arr(Vec<SifVal>),
    Tab(HashMap<String, SifVal>),
    Null,
}

impl fmt::Display for SifVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SifVal::Num(numval) => write!(f, "{}", numval),
            SifVal::Str(strval) => write!(f, "{}", strval),
            SifVal::Bl(blval) => write!(f, "{}", blval),
            SifVal::Arr(valvec) => {
                let mut contents = String::from("[");
                for (i, val) in valvec.iter().enumerate() {
                    contents.push_str(&format!("{:#}", val));
                    if i != valvec.len() - 1 {
                        contents.push_str(", ");
                    }
                }
                contents.push_str("]");
                write!(f, "{}", contents)
            }
            SifVal::Tab(map) => {
                let mut contents = String::from("{");
                for (key, val) in map.iter() {
                    contents.push_str(&format!("{:#}: ", key));
                    contents.push_str(&format!("{:#}, ", val));
                }
                contents.pop();
                contents.pop();
                contents.push_str("}");
                write!(f, "{:?}", contents)
            }
            SifVal::Null => write!(f, "{}", "null"),
        }
    }
}
