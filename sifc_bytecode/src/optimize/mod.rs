mod redundant_jmp;

use crate::instr::Instr;
use crate::optimize::redundant_jmp::RedundantJmp;

pub trait BytecodePass<'b> {
    fn name(&self) -> String;
    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr>;
}

pub struct OptimizeResult {
    pub optimized: Vec<Instr>,
}

pub struct BytecodeOptimizer {
    redundant_jmp: RedundantJmp,
}

impl BytecodeOptimizer {
    pub fn new() -> BytecodeOptimizer {
        BytecodeOptimizer {
            redundant_jmp: RedundantJmp,
        }
    }

    pub fn run_passes(&self, prog: &Vec<Instr>) -> OptimizeResult {
        let r1 = self.redundant_jmp.run_pass(prog);
        OptimizeResult { optimized: r1 }
    }
}
