use crate::{
    instr::Instr,
    opc::{Op, OpTy},
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
                ty,
                src1,
                src2,
                dest,
            } => {
                let op_str = op_ty_str(ty);
                let reg1 = reg_str(src1);
                let reg2 = reg_str(src2);
                let dstr = reg_str(dest);
                let line = format!("\t{} {} {} {}\n", op_str, reg1, reg2, dstr);
                dble.push_str(&line);
            }
            Op::Unary { ty, src1, dest } => {
                let op_str = op_ty_str(ty);
                let reg1 = reg_str(src1);
                let dstr = reg_str(dest);
                let line = format!("\t{} {} {}\n", op_str, reg1, dstr);
                dble.push_str(&line);
            }
            Op::LoadC { ty, dest, val } => {
                let op_str = op_ty_str(ty);
                let dstr = reg_str(dest);
                let vstr = val_str(val);
                let line = format!("\t{} {} {}\n", op_str, vstr, dstr);
                dble.push_str(&line);
            }
            Op::LoadN { ty, dest, name } => {
                let op_str = op_ty_str(ty);
                let dstr = reg_str(dest);
                let line = format!("\t{} {} {}\n", op_str, name, dstr);
                dble.push_str(&line);
            }
            Op::StoreC { ty, name, val } => {
                let op_str = op_ty_str(ty);
                let vstr = val_str(val);
                let line = format!("\t{} {} {}\n", op_str, vstr, name);
                dble.push_str(&line);
            }
            Op::StoreN {
                ty,
                srcname,
                destname,
            } => {
                let op_str = op_ty_str(ty);
                let line = format!("\t{} {} {}\n", op_str, srcname, destname);
                dble.push_str(&line);
            }
            Op::StoreR { ty, name, src } => {
                let op_str = op_ty_str(ty);
                let rstr = reg_str(src);
                let line = format!("\t{} {} {}\n", op_str, name, rstr);
                dble.push_str(&line);
            }
            Op::JumpCnd { ty, src, lbl } => {
                let op_str = op_ty_str(ty);
                let rstr = reg_str(src);
                let line = format!("\t{} {} {}\n", op_str, rstr, lbl);
                dble.push_str(&line);
            }
            Op::JumpA { ty, lbl } => {
                let op_str = op_ty_str(ty);
                let line = format!("\t{} {}\n", op_str, lbl);
                dble.push_str(&line);
            }
            Op::Nop { ty } => {
                let op_str = op_ty_str(ty);
                let line = format!("\t{}\n", op_str);
                dble.push_str(&line);
            }
            Op::Incrr { ty, src } => {
                let op_str = op_ty_str(ty);
                let rstr = reg_str(src);
                let line = format!("\t{} {}\n", op_str, rstr);
                dble.push_str(&line);
            }
            Op::Decrr { ty, src } => {
                let op_str = op_ty_str(ty);
                let rstr = reg_str(src);
                let line = format!("\t{} {}\n", op_str, rstr);
                dble.push_str(&line);
            }
        }
    }

    println!("{}", dble);
}

fn op_ty_str(opty: OpTy) -> &'static str {
    match opty {
        OpTy::Add => "add",
        OpTy::Sub => "sub",
        OpTy::Mul => "mul",
        OpTy::Div => "div",
        OpTy::Modu => "mod",
        OpTy::Eq => "eq",
        OpTy::Neq => "neq",
        OpTy::LtEq => "lteq",
        OpTy::Lt => "lt",
        OpTy::GtEq => "gteq",
        OpTy::Gt => "gt",
        OpTy::Land => "and",
        OpTy::Lnot => "not",
        OpTy::Lor => "or",
        OpTy::Lneg | OpTy::Nneg => "neg",
        OpTy::Ldc => "ldc",
        OpTy::Ldn => "ldn",
        OpTy::Stc => "stc",
        OpTy::Stn => "stn",
        OpTy::Str => "str",
        OpTy::Jmpt => "jmpt",
        OpTy::Jmpf => "jmpf",
        OpTy::Jmp => "jmp",
        OpTy::Incrr => "incrr",
        OpTy::Decrr => "decrr",
        OpTy::Stop => "stp",
        OpTy::Nop => "nop",
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
