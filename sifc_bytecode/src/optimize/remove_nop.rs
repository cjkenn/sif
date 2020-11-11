use crate::optimize::bco::BytecodePass;
use crate::{instr::Instr, opc::Op};

/// Removes any nop instructions that are not the only instruction in a block. We have to leave
/// any that are sole block instructions because removing them might mess up jump targets.
pub struct RemoveNop;

impl<'b> BytecodePass<'b> for RemoveNop {
    fn name(&self) -> String {
        String::from("RemoveNop")
    }

    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr> {
        let mut result = Vec::new();
        result.push(bytecode[0].clone());

        let mut i = 1;
        while i < bytecode.len() - 1 {
            let curr = &bytecode[i];
            let prev = &bytecode[i - 1];
            let next = &bytecode[i + 1];

            match curr.op {
                Op::Nop => {
                    // If the nop is in it's own block (or happens to be the last instruction),
                    // we should include it in the optimized result.
                    if curr.lblidx != prev.lblidx && curr.lblidx != next.lblidx {
                        result.push(curr.clone());
                    }
                }
                _ => result.push(curr.clone()),
            }
            i += 1;
        }

        result.push(bytecode[i].clone());
        result
    }
}
