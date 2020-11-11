use crate::optimize::bco::BytecodePass;
use crate::{instr::Instr, opc::Op};

/// If there are any remaining instructions in a block after a return instruction,
/// we can remove them. Note that these instructions won't be executed anyway because
/// the ret call will transfer control, but we can save size in blocks which can increase
/// the speed of further analysis (and save memory, but that's likely negligable).
pub struct RemoveAfterRet;

impl<'b> BytecodePass<'b> for RemoveAfterRet {
    fn name(&self) -> String {
        String::from("RemoveAfterRet")
    }

    fn run_pass(&self, bytecode: &'b Vec<Instr>) -> Vec<Instr> {
        let mut i = 0;
        let mut result = Vec::new();

        while i < bytecode.len() {
            let instr = &bytecode[i];
            let currlbl = instr.lblidx;
            match instr.op {
                Op::FnRet => {
                    // At a return instruction, we look ahead to any further
                    // instructions in the block and skip past them so they
                    // won't be added to the result array.
                    let mut j = i;
                    while bytecode[j].lblidx == currlbl {
                        j += 1;
                        if j == bytecode.len() {
                            break;
                        }
                    }
                    i = j - 1;
                }
                _ => {}
            };

            result.push(instr.clone());
            i += 1;
        }

        result
    }
}
