use crate::{
    instr::Instr,
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};

/// Prints the declaration section to stdout.
pub fn dump_decls(decls: Vec<Instr>) {
    if decls.len() == 0 {
        return;
    }

    let mut dble = String::from("SECTION_ decls.\n");

    dump(decls, &mut dble)
}

/// Prints the code section to stdout.
pub fn dump_code(code: Vec<Instr>) {
    if code.len() == 0 {
        return;
    }

    let mut dble = String::from("SECTION_ code.\n");
    let currlbl = code[0].lbl.clone();
    dble.push_str(&format!("{}:\n", currlbl));

    dump(code, &mut dble);
}

/// dump will parse the vector of instrs and transform it into typical
/// asm-looking strings for printing. We choose not to override the Debug
/// and Display traits as they can still be useful for pretty printing
/// the actual structs and vectors at other times, as this method
/// does not contain all the information held in those structs.
fn dump(ir: Vec<Instr>, dble: &mut String) {
    let mut currlbl = ir[0].lbl.clone();
    for i in ir {
        if i.lbl != currlbl {
            let line = format!("{}:\n", &i.lbl);
            dble.push_str(&line);
            currlbl = i.lbl.clone();
        }

        dble.push_str(&format!("{:#?}", i));
    }

    println!("{}", dble);
}
