use crate::{cfg, ssa::builder::SSABuilder};
use sifc_bytecode::instr::Instr;

pub struct Analyzer {
    program: Vec<Instr>,
}

impl Analyzer {
    pub fn new(v: Vec<Instr>) -> Analyzer {
        Analyzer { program: v }
    }

    pub fn perform(&self) {
        let cfg = cfg::build_cfg(&self.program);
        let ssab = SSABuilder::new(cfg);
        ssab.build();
    }
}