use crate::op::OpCode;

pub struct SifBlock {
    instrs: Vec<OpCode>,
}

impl SifBlock {
    pub fn new() -> SifBlock {
        SifBlock { instrs: Vec::new() }
    }

    pub fn write(&mut self, op: OpCode) {
        self.instrs.push(op);
    }
}
