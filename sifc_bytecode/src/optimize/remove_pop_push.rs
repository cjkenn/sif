use crate::optimize::bco::BytecodePass;
use crate::{instr::Instr, opc::Op};

/// If there are any pairs of instructions where we pop a value from the function stack into
/// a register and then immediately push that value back onto the stack, we can simply remove
/// those since they undo each other. We must make sure the popped/pushed value is the same
/// (or comes from the same register).
pub struct RemovePopPush;

impl<'b> BytecodePass<'b> for RemovePopPush {
    fn name(&self) -> String {
        String::from("RemovePopPush")
    }

    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr> {
        let mut i = 0;
        let mut result = Vec::new();

        while i < bytecode.len() - 1 {
            let instr = &bytecode[i];
            let next = &bytecode[i + 1];

            match instr.op {
                Op::FnStackPop { dest } => {
                    match next.op {
                        Op::FnStackPush { src } => {
                            if dest != src {
                                result.push(instr.clone());
                            } else {
                                i += 1; // we also skip this push instruction, so increment i
                            }
                        }
                        _ => result.push(instr.clone()),
                    };
                }
                _ => result.push(instr.clone()),
            };

            i += 1;
        }

        result.push(bytecode[i].clone());
        result
    }
}
