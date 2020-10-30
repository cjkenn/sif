use crate::optimize::BytecodePass;
use crate::{instr::Instr, opc::Op};

pub struct RedundantJmp;

impl<'b> BytecodePass<'b> for RedundantJmp {
    fn name(&self) -> String {
        String::from("RedundantJmp")
    }

    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr> {
        // If a jump goes to the immediate next label and is the last instruction
        // under the current label, remove it.
        let mut i = 0;
        let mut result = Vec::new();

        while i < bytecode.len() - 1 {
            let instr = &bytecode[i];
            let next_lbl = bytecode[i + 1].lblidx;

            if instr.lblidx != next_lbl {
                let is_safe = match instr.op {
                    Op::JumpCnd {
                        kind: _,
                        src: _,
                        lblidx,
                    } => lblidx != next_lbl,
                    Op::JumpA { lblidx } => lblidx != next_lbl,
                    _ => true,
                };
                if is_safe {
                    result.push(instr.clone());
                }
            } else {
                result.push(instr.clone());
            }

            i += 1;
        }

        result.push(bytecode[bytecode.len() - 1].clone());
        result
    }
}
