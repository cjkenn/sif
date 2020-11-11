use crate::optimize::bco::BytecodePass;
use crate::{instr::Instr, opc::Op};

/// If a jmp instruction transfers control to the immediate next instruction,
/// we can remove it and allow the program execution to continue.
pub struct RedundantJmp;

impl<'b> BytecodePass<'b> for RedundantJmp {
    fn name(&self) -> String {
        String::from("RedundantJmp")
    }

    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr> {
        let mut i = 0;
        let mut result = Vec::new();

        while i < bytecode.len() - 1 {
            let instr = &bytecode[i];
            let next_lbl = bytecode[i + 1].lblidx;

            if instr.lblidx != next_lbl {
                let is_safe = match instr.op {
                    Op::JmpCnd {
                        kind: _,
                        src: _,
                        lblidx,
                    } => lblidx != next_lbl,
                    Op::Jmpa { lblidx } => lblidx != next_lbl,
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
