use crate::{instr::Instr, opc::Op};
use std::collections::HashMap;

pub fn compute(program: &Vec<Instr>) -> (HashMap<usize, usize>, HashMap<String, usize>) {
    let mut jt = HashMap::new();
    let mut ft = HashMap::new();
    if program.len() == 0 {
        return (jt, ft);
    }

    jt.insert(0, 0);
    let mut curridx = program[0].lblidx;

    for (i, instr) in program.iter().enumerate() {
        if instr.lblidx != curridx {
            curridx = instr.lblidx;
            jt.insert(instr.lblidx, i);
        }

        match &instr.op {
            Op::Fn { name, .. } => {
                ft.insert(name.clone(), i);
            }
            _ => {}
        };
    }

    (jt, ft)
}
