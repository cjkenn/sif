#[derive(Debug, Clone)]
pub enum SifVal {
    Num(f64),
    Str(String),
    Bl(bool),
    Null,
}
