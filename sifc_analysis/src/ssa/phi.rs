use crate::block::SifBlockRef;

#[derive(Debug, Clone, PartialEq)]
pub struct PhiFn {
    pub dest: String,
    pub operands: Vec<String>,
}

impl PhiFn {
    pub fn new(d: String, o: Vec<String>) -> PhiFn {
        PhiFn {
            dest: d,
            operands: o,
        }
    }
}
