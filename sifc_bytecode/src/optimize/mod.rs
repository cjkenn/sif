mod redundant_jmp;
mod remove_after_ret;
mod remove_nop;

use crate::instr::Instr;
use crate::optimize::{
    redundant_jmp::RedundantJmp, remove_after_ret::RemoveAfterRet, remove_nop::RemoveNop,
};

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
    remove_nop: RemoveNop,
}

impl BytecodeOptimizer {
    pub fn new() -> BytecodeOptimizer {
        BytecodeOptimizer {
            redundant_jmp: RedundantJmp,
            remove_after_ret: RemoveAfterRet,
            remove_nop: RemoveNop,
        }
    }

    pub fn run_passes(&self, prog: &Vec<Instr>) -> OptimizeResult {
        let r1 = self.redundant_jmp.run_pass(prog);
        let r2 = self.remove_after_ret.run_pass(&r1);
        let r3 = self.remove_nop.run_pass(&r2);

        OptimizeResult { optimized: r3 }
    }
}
