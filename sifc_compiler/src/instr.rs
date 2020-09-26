use crate::opc::Op;

#[derive(Debug, Clone)]
pub struct Instr {
    pub lbl: String,
    pub op: Op,
}

impl Instr {
    pub fn new(l: String, o: Op) -> Instr {
        Instr { lbl: l, op: o }
    }
}
