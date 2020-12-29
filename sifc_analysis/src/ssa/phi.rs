use crate::ssa::SSAVal;

#[derive(Debug, Clone, PartialEq)]
pub struct PhiFn {
    pub operands: Vec<SSAVal>,
}

impl PhiFn {
    pub fn new() -> PhiFn {
        PhiFn {
            operands: Vec::new(),
        }
    }
}
