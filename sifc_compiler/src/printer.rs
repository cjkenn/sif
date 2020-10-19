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
pub fn dump(ir: Vec<Instr>, name: &str) {
    if ir.len() == 0 {
        return;
    }

    let mut dble = format!("SECTION_ {}\n", name);

    // TODO: dont do this for non-code sections
    let mut currlbl = ir[0].lbl.clone();
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
                let line = format!("\t {} {} {} {}\t ; {}\n", op_str, reg1, reg2, dstr, i.line);
                dble.push_str(&line);
            }
            Op::Unary { kind, src1, dest } => {
                let op_str = un_kind_str(kind);
                let reg1 = reg_str(src1);
                let dstr = reg_str(dest);
                let line = format!("\t {} {} {}\t ; {}\n", op_str, reg1, dstr, i.line);
                dble.push_str(&line);
            }
            Op::LoadC { dest, val } => {
                let dstr = reg_str(dest);
                let vstr = val_str(val);
                let line = format!("\t ldc {} {}\t ; {}\n", vstr, dstr, i.line);
                dble.push_str(&line);
            }
            Op::LoadN { dest, name } => {
                let dstr = reg_str(dest);
                let line = format!("\t ldn {} {}\t ; {}\n", name, dstr, i.line);
                dble.push_str(&line);
            }
            Op::LoadArrs { name, dest } => {
                let dstr = reg_str(dest);
                let line = format!("\t ldarrs {} {}\t ; {}\n", name, dstr, i.line);
                dble.push_str(&line);
            }
            Op::LoadArrv { name, idx, dest } => {
                let dstr = reg_str(dest);
                let istr = reg_str(idx);
                let line = format!("\t ldarrv {} {} {}\t ; {}\n", name, istr, dstr, i.line);
                dble.push_str(&line);
            }
            Op::StoreC { name, val } => {
                let vstr = val_str(val);
                let line = format!("\t stc {} {}\t ; {}\n", vstr, name, i.line);
                dble.push_str(&line);
            }
            Op::StoreN { srcname, destname } => {
                let line = format!("\t stn {} {}\t ; {}\n", srcname, destname, i.line);
                dble.push_str(&line);
            }
            Op::StoreR { name, src } => {
                let rstr = reg_str(src);
                let line = format!("\t strr {} {}\t ; {}\n", rstr, name, i.line);
                dble.push_str(&line);
            }
            Op::JumpCnd { kind, src, lbl, .. } => {
                let op_str = jmp_kind_str(kind);
                let rstr = reg_str(src);
                let line = format!("\t {} {} {}\t ; {}\n", op_str, rstr, lbl, i.line);
                dble.push_str(&line);
            }
            Op::JumpA { lbl, .. } => {
                let line = format!("\t jmpa {}\t ; {}\n", lbl, i.line);
                dble.push_str(&line);
            }
            Op::Nop => {
                let line = format!("\t {}\t\t ; {}\n", "nop", i.line);
                dble.push_str(&line);
            }
            Op::Incrr { src } => {
                let rstr = reg_str(src);
                let line = format!("\t incrr {}\t ; {}\n", rstr, i.line);
                dble.push_str(&line);
            }
            Op::Decrr { src } => {
                let rstr = reg_str(src);
                let line = format!("\t decrr {}\t ; {}\n", rstr, i.line);
                dble.push_str(&line);
            }
            Op::Fn { name, params } => {
                let line = format!("fn @{} {:?}\n", name, params);
                dble.push_str(&line);
            }
            Op::FnRetR { src } => {
                let rstr = reg_str(src);
                let line = format!("\t retr {}\t ; {}\n", rstr, i.line);
                dble.push_str(&line);
            }
            Op::FnRet => {
                let line = format!("\t ret \t\t ; {} \n", i.line);
                dble.push_str(&line);
            }
            Op::Call { name } => {
                let line = format!("\t call {}\t ; {} \n", name, i.line);
                dble.push_str(&line);
            }
            Op::Stop => {
                let line = format!("\t{}\t\t ; {}\n", "stop", i.line);
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
