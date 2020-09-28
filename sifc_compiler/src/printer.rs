use crate::instr::Instr;
use crate::opc::OpTy;

/// dump will parse the vector of instrs and transform it into typical
/// asm-looking strings for printing. We choose not to override the Debug
/// and Display traits as they can still be useful for pretty printing
/// the actual structs and vectors at other times, as this method
/// does not contain all the information held in those structs.
pub fn dump(ir: Vec<Instr>) {
    if ir.len() == 0 {
        return;
    }

    let mut currlbl = ir[0].lbl.clone();
    let mut dble = String::new();

    for i in ir {
        if i.lbl != currlbl {
            dble.push_str(&i.lbl);
            currlbl = i.lbl;
        }
    }

    println!("{}", dble);
}

fn op_ty_str(opty: OpTy) -> &'static str {
    ""
}
