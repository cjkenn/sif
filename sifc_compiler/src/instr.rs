use crate::opc::Opc;

#[derive(Debug, Clone)]
pub struct Instr<'i> {
    pub lbl: String,
    pub op: Opc<'i>,
}

impl<'i> Instr<'i> {
    pub fn new(l: String, o: Opc<'i>) -> Instr<'i> {
        Instr { lbl: l, op: o }
    }
}
