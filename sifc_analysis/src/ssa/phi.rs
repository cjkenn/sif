use crate::{cfg::SifBlockRef, ssa::SSAVal};

#[derive(Debug, Clone, PartialEq)]
pub struct PhiFn {
    pub block: SifBlockRef,
    pub operands: Vec<SSAVal>,
}

impl PhiFn {
    pub fn new(block: SifBlockRef) -> PhiFn {
        PhiFn {
            block: block,
            operands: Vec::new(),
        }
    }
}
