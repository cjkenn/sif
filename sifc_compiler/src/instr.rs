use crate::opc::Opc;

#[derive(Debug, Clone)]
pub struct Instr {
    pub lbl: String,
    pub op: Opc,
}

impl Instr {
    pub fn new(l: String, o: Opc) -> Instr {
        Instr { lbl: l, op: o }
    }
}
