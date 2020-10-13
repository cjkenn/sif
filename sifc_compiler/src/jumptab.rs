use crate::{
    instr::Instr,
    opc::{Op, OpTy},
    sifv::SifVal,
};
use std::collections::HashMap;

/// Returns a map from usize to usize. The key is the index of the label,
/// and the value is the index of the first instruction in the code vector
/// under that label.
pub fn compute_jumptab(ops: &Vec<Instr>) -> HashMap<usize, usize> {
    let mut map = HashMap::new();
    if ops.len() == 0 {
        return map;
    }

    map.insert(0, 0);
    let mut curridx = ops[0].lblidx;

    for (i, op) in ops.iter().enumerate() {
        if op.lblidx != curridx {
            curridx = op.lblidx;
            map.insert(op.lblidx, i);
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_on_no_code() {
        let res = compute_jumptab(&Vec::new());

        assert_eq!(0, res.len());
    }

    #[test]
    fn single_instr() {
        let mut code = Vec::new();
        code.push(Instr::new(0, Op::Nop, 1));
        let res = compute_jumptab(&code);

        assert_eq!(1, res.len());

        let g = res.get(&0).unwrap();
        assert_eq!(*g, 0);
        assert_eq!(res.get(&1), None);
    }

    #[test]
    fn branching_instrs() {
        let code = build_branch_code();
        let res = compute_jumptab(&code);

        assert_eq!(3, res.len());

        let first = res.get(&0).unwrap();
        assert_eq!(*first, 0);

        let second = res.get(&1).unwrap();
        assert_eq!(*second, 4);

        let third = res.get(&2).unwrap();
        assert_eq!(*third, 5);

        assert_eq!(res.get(&3), None);
    }

    // Builds the following instructions:
    // lbl0:
    // 	ldc 1 r0
    // 	ldc 2 r1
    // 	lt r0 r1 r2
    // 	jmpf r2 lbl2
    // lbl1:
    // 	jmp lbl2
    // lbl2:
    // 	nop
    fn build_branch_code() -> Vec<Instr> {
        let mut code = Vec::new();

        code.push(Instr::new(
            0,
            Op::LoadC {
                ty: OpTy::Ldc,
                dest: 0,
                val: SifVal::Num(1.0),
            },
            1,
        ));

        code.push(Instr::new(
            0,
            Op::LoadC {
                ty: OpTy::Ldc,
                dest: 1,
                val: SifVal::Num(2.0),
            },
            2,
        ));

        code.push(Instr::new(
            0,
            Op::Binary {
                ty: OpTy::Lt,
                src1: 0,
                src2: 1,
                dest: 2,
            },
            3,
        ));

        code.push(Instr::new(
            0,
            Op::JumpCnd {
                ty: OpTy::Jmpf,
                src: 2,
                lbl: "lbl2".to_string(),
                lblidx: 2,
            },
            4,
        ));

        code.push(Instr::new(
            1,
            Op::JumpA {
                ty: OpTy::Jmp,
                lbl: "lbl2".to_string(),
                lblidx: 2,
            },
            5,
        ));

        code.push(Instr::new(2, Op::Nop, 6));

        code
    }
}
