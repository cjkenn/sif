mod redundant_jmp;
mod remove_after_ret;
mod remove_nop;

use crate::instr::Instr;
use crate::optimize::{
    redundant_jmp::RedundantJmp, remove_after_ret::RemoveAfterRet, remove_nop::RemoveNop,
};
use std::collections::HashMap;

pub trait BytecodePass<'b> {
    fn name(&self) -> String;
    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr>;
}

pub struct OptimizeResult {
    pub new_code_start: usize,
    pub optimized: Vec<Instr>,
    pub removed: usize,
    pub jumptab: HashMap<usize, usize>,
    pub fntab: HashMap<String, usize>,
}

struct InternalPassResult {
    pub optimized: Vec<Instr>,
    pub removed: usize,
}

pub struct BytecodeOptimizer {
    init_code_start: usize,
    decls: Vec<Instr>,
    code: Vec<Instr>,

    redundant_jmp: RedundantJmp,
    remove_after_ret: RemoveAfterRet,
    remove_nop: RemoveNop,
}

impl BytecodeOptimizer {
    pub fn new(decls: Vec<Instr>, code: Vec<Instr>, code_start: usize) -> BytecodeOptimizer {
        BytecodeOptimizer {
            init_code_start: code_start,
            decls: decls,
            code: code,
            redundant_jmp: RedundantJmp,
            remove_after_ret: RemoveAfterRet,
            remove_nop: RemoveNop,
        }
    }

    pub fn run_passes(&mut self) -> OptimizeResult {
        let decls_result = self.optimize_decls();
        let code_result = self.optimize_code();

        let instrs_removed = decls_result.removed + code_result.removed;
        let code_start = self.init_code_start - decls_result.removed;

        let mut full_prog = decls_result.optimized.clone();
        full_prog.extend(code_result.optimized.iter().cloned());

        let (jumptab, fntab) = crate::tables::compute(&full_prog);

        OptimizeResult {
            new_code_start: code_start,
            removed: instrs_removed,
            optimized: full_prog,
            jumptab: jumptab,
            fntab: fntab,
        }
    }

    fn optimize_decls(&mut self) -> InternalPassResult {
        let r1 = self.redundant_jmp.run_pass(&self.decls);
        let r2 = self.remove_after_ret.run_pass(&r1);
        let r3 = self.remove_nop.run_pass(&r2);

        InternalPassResult {
            removed: self.decls.len() - r3.len(),
            optimized: r3,
        }
    }

    fn optimize_code(&mut self) -> InternalPassResult {
        let r1 = self.redundant_jmp.run_pass(&self.code);
        let r2 = self.remove_after_ret.run_pass(&r1);
        let r3 = self.remove_nop.run_pass(&r2);

        InternalPassResult {
            removed: self.code.len() - r3.len(),
            optimized: r3,
        }
    }
}
