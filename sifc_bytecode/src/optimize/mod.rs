mod redundant_jmp;
mod remove_after_ret;

use crate::instr::Instr;
use crate::optimize::{redundant_jmp::RedundantJmp, remove_after_ret::RemoveAfterRet};

pub trait BytecodePass<'b> {
    fn name(&self) -> String;
    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr>;
}

pub struct OptimizeResult {
    pub optimized: Vec<Instr>,
}

pub struct BytecodeOptimizer {
    redundant_jmp: RedundantJmp,
    remove_after_ret: RemoveAfterRet,
}

impl BytecodeOptimizer {
    pub fn new() -> BytecodeOptimizer {
        BytecodeOptimizer {
            redundant_jmp: RedundantJmp,
            remove_after_ret: RemoveAfterRet,
        }
    }

    pub fn run_passes(&self, prog: &Vec<Instr>) -> OptimizeResult {
        let r1 = self.redundant_jmp.run_pass(prog);
        let r2 = self.remove_after_ret.run_pass(&r1);

        OptimizeResult { optimized: r2 }
    }
}
