use crate::opc::Op;

#[derive(Debug, Clone)]
pub struct Instr {
    pub lbl: String,
    pub lblidx: usize,
    pub op: Op,
}

impl Instr {
    pub fn new(l: String, i: usize, o: Op) -> Instr {
        Instr {
            lbl: l,
            lblidx: i,
            op: o,
        }
    }
}
