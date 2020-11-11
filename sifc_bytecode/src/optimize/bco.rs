use crate::instr::Instr;
use crate::optimize::{
    redundant_jmp::RedundantJmp, remove_after_ret::RemoveAfterRet, remove_nop::RemoveNop,
};
use std::collections::HashMap;

/// Implement this trait when writing optimize passes.
pub trait BytecodePass<'b> {
    fn name(&self) -> String;
    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr>;
}

/// Returns the optimized program as well as additional runtime information. Because instructions
/// can be added/removed and reordered, various indices returned from the compiler need to be
/// recalculated. Thus, this struct somewhat mirrors the CompileResult struct in its information.
pub struct OptimizeResult {
    /// Index indicating the start of the code section. This is recalculated from the amount of
    /// instructions changed in the decl section.
    pub new_code_start: usize,

    /// Fully optimized program.
    pub optimized: Vec<Instr>,

    /// Total amount of instructions removed from any section.
    pub removed: usize,

    /// Newly calculated jump table. See CompileResult for more detailed info.
    pub jumptab: HashMap<usize, usize>,

    /// Newly calculated function table. See CompileResult for more detailed info.
    pub fntab: HashMap<String, usize>,
}

/// Used to hold intermediate results of various passes.
struct SectionPassResult {
    pub optimized: Vec<Instr>,
    pub removed: usize,
}

/// The BytecodeOptimizer takes in program declarations and code and runs a series of passes on them,
/// primarily for the purpose of removing redundant or unneeded instructions to save execution
/// passes in the vm. Instructions could also be re-ordered or changed to different types depending
/// on performance.
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

    fn optimize_decls(&mut self) -> SectionPassResult {
        let r1 = self.redundant_jmp.run_pass(&self.decls);
        let r2 = self.remove_after_ret.run_pass(&r1);
        let r3 = self.remove_nop.run_pass(&r2);

        SectionPassResult {
            removed: self.decls.len() - r3.len(),
            optimized: r3,
        }
    }

    fn optimize_code(&mut self) -> SectionPassResult {
        let r1 = self.redundant_jmp.run_pass(&self.code);
        let r2 = self.remove_after_ret.run_pass(&r1);
        let r3 = self.remove_nop.run_pass(&r2);

        SectionPassResult {
            removed: self.code.len() - r3.len(),
            optimized: r3,
        }
    }
}
