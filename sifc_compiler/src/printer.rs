use crate::{
    instr::Instr,
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};

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
    dble.push_str(&format!("{}:\n", currlbl));

    for i in ir {
        if i.lbl != currlbl {
            let line = format!("{}:\n", &i.lbl);
            dble.push_str(&line);
            currlbl = i.lbl;
        }

        match i.op {
            Op::Binary {
                kind,
                src1,
                src2,
                dest,
            } => {
                let op_str = bin_kind_str(kind);
                let reg1 = reg_str(src1);
                let reg2 = reg_str(src2);
                let dstr = reg_str(dest);
                let line = format!("{}. \t{} {} {} {}\n", i.line, op_str, reg1, reg2, dstr);
                dble.push_str(&line);
            }
            Op::Unary { kind, src1, dest } => {
                let op_str = un_kind_str(kind);
                let reg1 = reg_str(src1);
                let dstr = reg_str(dest);
                let line = format!("{}. \t{} {} {}\n", i.line, op_str, reg1, dstr);
                dble.push_str(&line);
            }
            Op::LoadC { dest, val } => {
                let dstr = reg_str(dest);
                let vstr = val_str(val);
                let line = format!("{}. \tldc {} {}\n", i.line, vstr, dstr);
                dble.push_str(&line);
            }
            Op::LoadN { dest, name } => {
                let dstr = reg_str(dest);
                let line = format!("{}. \tldn {} {}\n", i.line, name, dstr);
                dble.push_str(&line);
            }
            Op::LoadArrs { name, dest } => {
                let dstr = reg_str(dest);
                let line = format!("{}. \tldarrs {} {}\n", i.line, name, dstr);
                dble.push_str(&line);
            }
            Op::LoadArrv { name, idx, dest } => {
                let dstr = reg_str(dest);
                let istr = reg_str(idx);
                let line = format!("{}. \tldarrv {} {} {}\n", i.line, name, istr, dstr);
                dble.push_str(&line);
            }
            Op::StoreC { name, val } => {
                let vstr = val_str(val);
                let line = format!("{}. \tstc {} {}\n", i.line, vstr, name);
                dble.push_str(&line);
            }
            Op::StoreN { srcname, destname } => {
                let line = format!("{}. \tstn {} {}\n", i.line, srcname, destname);
                dble.push_str(&line);
            }
            Op::StoreR { name, src } => {
                let rstr = reg_str(src);
                let line = format!("{}. \tstrr {} {}\n", i.line, rstr, name);
                dble.push_str(&line);
            }
            Op::JumpCnd { kind, src, lbl, .. } => {
                let op_str = jmp_kind_str(kind);
                let rstr = reg_str(src);
                let line = format!("{}. \t{} {} {}\n", i.line, op_str, rstr, lbl);
                dble.push_str(&line);
            }
            Op::JumpA { lbl, .. } => {
                let line = format!("{}. \tjmpa {}\n", i.line, lbl);
                dble.push_str(&line);
            }
            Op::Nop => {
                let line = format!("{}. \t{}\n", i.line, "nop");
                dble.push_str(&line);
            }
            Op::Incrr { src } => {
                let rstr = reg_str(src);
                let line = format!("{}. \tincrr {}\n", i.line, rstr);
                dble.push_str(&line);
            }
            Op::Decrr { src } => {
                let rstr = reg_str(src);
                let line = format!("{}. \tdecrr {}\n", i.line, rstr);
                dble.push_str(&line);
            }
            Op::Stop => {
                let line = format!("{}. \t{}\n", i.line, "stop");
                dble.push_str(&line);
            }
        }
    }

    println!("{}", dble);
}

fn bin_kind_str(kind: BinOpKind) -> &'static str {
    match kind {
        BinOpKind::Add => "add",
        BinOpKind::Sub => "sub",
        BinOpKind::Mul => "mul",
        BinOpKind::Div => "div",
        BinOpKind::Modu => "mod",
        BinOpKind::Eq => "eq",
        BinOpKind::Neq => "neq",
        BinOpKind::LtEq => "lteq",
        BinOpKind::Lt => "lt",
        BinOpKind::GtEq => "gteq",
        BinOpKind::Gt => "gt",
        BinOpKind::Land => "and",
        BinOpKind::Lnot => "not",
        BinOpKind::Lor => "or",
    }
}

fn un_kind_str(kind: UnOpKind) -> &'static str {
    match kind {
        UnOpKind::Lneg => "lneg",
        UnOpKind::Nneg => "nneg",
    }
}

fn jmp_kind_str(kind: JmpOpKind) -> &'static str {
    match kind {
        JmpOpKind::Jmpt => "jmpt",
        JmpOpKind::Jmpf => "jmpf",
    }
}

fn reg_str(reg: usize) -> String {
    format!("r{}", reg)
}

fn val_str(v: SifVal) -> String {
    match v {
        SifVal::Num(v) => v.to_string(),
        SifVal::Str(s) => s,
        SifVal::Bl(b) => b.to_string(),
        SifVal::Null => "null".to_string(),
        SifVal::Arr(a) => format!("{:?}", a),
    }
}
