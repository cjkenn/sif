use crate::block::SifBlockRef;

#[derive(Debug, Clone, PartialEq)]
pub struct PhiFn {
    pub operands: Vec<SifBlockRef>,
}

impl PhiFn {
    pub fn new() -> PhiFn {
        PhiFn {
            operands: Vec::new(),
        }
    }

    pub fn from_operands(ops: Vec<SifBlockRef>) -> PhiFn {
        PhiFn { operands: ops }
    }
}
