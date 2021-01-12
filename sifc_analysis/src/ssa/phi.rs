#[derive(Debug, Clone, PartialEq)]
pub struct PhiFn {
    pub initial: String,
    pub dest: String,
    pub operands: Vec<PhiOp>,
}

impl PhiFn {
    pub fn new(i: String, d: String, o: Vec<PhiOp>) -> PhiFn {
        PhiFn {
            initial: i,
            dest: d,
            operands: o,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhiOp {
    pub slot: usize,
    pub name: String,
}

impl PhiOp {
    pub fn new(s: usize, n: String) -> PhiOp {
        PhiOp { slot: s, name: n }
    }
}
