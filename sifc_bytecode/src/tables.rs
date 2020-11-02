use crate::{instr::Instr, opc::Op};
use std::collections::HashMap;

/// Computes two branch tables from a generated program and returns them in a tuple:
/// 1. The first table in the tuple contains addresses of labels to jump to using
///    jmp instructions.
/// 2. The second table in the tuple contains addresses of fn decls to jump to using
///    call instructions.
/// Two tables are used because the addresses of functions are in a different program
/// section (declaration) than the jump addresses.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        opc::{BinOpKind, JmpOpKind, Op},
        sifv::SifVal,
    };

    #[test]
    fn test_compute_empty() {
        let prog = Vec::new();
        let (jt, ft) = compute(&prog);
        assert!(jt.len() == 0);
        assert!(ft.len() == 0);
    }

    #[test]
    fn single_instr() {
        let mut code = Vec::new();
        code.push(Instr::new(0, Op::Nop, 1));
        let (jt, ft) = compute(&code);

        assert!(jt.len() == 1);
        assert!(ft.len() == 0); // no fn decls

        let g = jt.get(&0).unwrap();
        assert_eq!(*g, 0);
        assert_eq!(jt.get(&1), None);
    }

    #[test]
    fn branching_instrs() {
        let code = build_branch_code();
        let (jt, ft) = compute(&code);

        assert!(jt.len() == 3);
        assert!(ft.len() == 0);

        let first = jt.get(&0).unwrap();
        assert_eq!(*first, 0);

        let second = jt.get(&1).unwrap();
        assert_eq!(*second, 4);

        let third = jt.get(&2).unwrap();
        assert_eq!(*third, 5);

        assert_eq!(jt.get(&3), None);
    }

    #[test]
    fn fn_decl_instrs() {
        let code = build_fn_code();
        let (jt, ft) = compute(&code);
        println!("{:#?}", jt);

        assert!(jt.len() == 1); // we always insert (0, 0)
        assert!(ft.len() == 1);

        let decl = ft.get("test");
        assert!(decl.is_some());
        assert_eq!(*decl.unwrap(), 0);
    }

    fn build_fn_code() -> Vec<Instr> {
        let mut code = Vec::new();

        code.push(Instr::new(
            0,
            Op::Fn {
                name: String::from("test"),
                params: Vec::new(),
            },
            1,
        ));

        code.push(Instr::new(0, Op::Nop, 2));
        code
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
            Op::Ldc {
                dest: 0,
                val: SifVal::Num(1.0),
            },
            1,
        ));

        code.push(Instr::new(
            0,
            Op::Ldc {
                dest: 1,
                val: SifVal::Num(2.0),
            },
            2,
        ));

        code.push(Instr::new(
            0,
            Op::Binary {
                kind: BinOpKind::Lt,
                src1: 0,
                src2: 1,
                dest: 2,
            },
            3,
        ));

        code.push(Instr::new(
            0,
            Op::JmpCnd {
                kind: JmpOpKind::Jmpf,
                src: 2,
                lblidx: 2,
            },
            4,
        ));

        code.push(Instr::new(1, Op::Jmpa { lblidx: 2 }, 5));
        code.push(Instr::new(2, Op::Nop, 6));

        code
    }
}
