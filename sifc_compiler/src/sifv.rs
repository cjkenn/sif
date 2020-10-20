use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum SifVal {
    Num(f64),
    Str(String),
    Bl(bool),
    Arr(Vec<SifVal>),
    Tab(HashMap<String, SifVal>),
    Null,
}
