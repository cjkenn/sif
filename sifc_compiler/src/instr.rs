use crate::opc::Op;

#[derive(Debug, Clone)]
pub struct Instr {
    pub lbl: String,
    pub lblidx: usize,
    pub op: Op,
    pub line: usize,
}

impl Instr {
    pub fn new(idx: usize, o: Op, l: usize) -> Instr {
        Instr {
            lbl: format!("lbl{}", idx),
            lblidx: idx,
            op: o,
            line: l,
        }
    }
}
