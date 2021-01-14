use crate::{cfg::CFG, ssa::builder::SSABuilder};
use sifc_bytecode::instr::Instr;

pub struct Analyzer {
    program: Vec<Instr>,
}

impl Analyzer {
    pub fn new(v: Vec<Instr>) -> Analyzer {
        Analyzer { program: v }
    }

    pub fn build_cfg(&self) -> CFG {
        CFG::build(&self.program)
    }

    pub fn build_ssa(&self) -> CFG {
        let cfg = CFG::build(&self.program);
        let mut ssab = SSABuilder::new(&cfg);
        ssab.build();
        cfg
    }

    pub fn perform(&self) {
        self.build_ssa();
    }
}
