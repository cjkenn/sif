use crate::{
    opc::{BinOpKind, JmpOpKind, Op, UnOpKind},
    sifv::SifVal,
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Instr {
    pub lbl: String,
    pub lblidx: usize,
    pub op: Op,
    pub line: usize,
}

impl Instr {
    pub fn new(idx: usize, o: Op, l: usize) -> Instr {
        Instr {
            lbl: format!("lbl{}", idx),
            lblidx: idx,
            op: o,
            line: l,
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut initial = String::new();
        initial.push_str(&format!("{}: ", self.lbl));

        match self.op.clone() {
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
                let line = format!("{} {} {} {}", op_str, reg1, reg2, dstr);
                initial.push_str(&line);
            }
            Op::Unary { kind, src1, dest } => {
                let op_str = un_kind_str(kind);
                let reg1 = reg_str(src1);
                let dstr = reg_str(dest);
                let line = format!("{} {} {}", op_str, reg1, dstr);
                initial.push_str(&line);
            }
            Op::LoadC { dest, val } => {
                let dstr = reg_str(dest);
                let vstr = val_str(val);
                let line = format!("ldc {} {}", vstr, dstr);
                initial.push_str(&line);
            }
            Op::LoadN { dest, name } => {
                let dstr = reg_str(dest);
                let line = format!("ldn {} {}", name, dstr);
                initial.push_str(&line);
            }
            Op::MvFRR { dest } => {
                let dstr = reg_str(dest);
                let line = format!("mvfrr {}", dstr);
                initial.push_str(&line);
            }
            Op::Mv { src, dest } => {
                let rstr = reg_str(src);
                let dstr = reg_str(dest);
                let line = format!("mv {} {}", rstr, dstr);
                initial.push_str(&line);
            }
            Op::LoadArrs { name, dest } => {
                let dstr = reg_str(dest);
                let line = format!("ldarrs {} {}", name, dstr);
                initial.push_str(&line);
            }
            Op::LoadArrv {
                name,
                idx_reg,
                dest,
            } => {
                let dstr = reg_str(dest);
                let istr = reg_str(idx_reg);
                let line = format!("ldarrv {} {} {}", name, istr, dstr);
                initial.push_str(&line);
            }
            Op::StoreC { name, val } => {
                let vstr = val_str(val);
                let line = format!("stc {} {}", vstr, name);
                initial.push_str(&line);
            }
            Op::StoreN { srcname, destname } => {
                let line = format!("stn {} {}", srcname, destname);
                initial.push_str(&line);
            }
            Op::StoreR { name, src } => {
                let rstr = reg_str(src);
                let line = format!("strr {} {}", rstr, name);
                initial.push_str(&line);
            }
            Op::StoreFRR { name } => {
                let line = format!("strfrr {}", name);
                initial.push_str(&line);
            }
            Op::JumpCnd { kind, src, lbl, .. } => {
                let op_str = jmp_kind_str(kind);
                let rstr = reg_str(src);
                let line = format!("{} {} {}", op_str, rstr, lbl);
                initial.push_str(&line);
            }
            Op::JumpA { lbl, .. } => {
                let line = format!("jmpa {}", lbl);
                initial.push_str(&line);
            }
            Op::Nop => {
                let line = format!("{}\t", "nop");
                initial.push_str(&line);
            }
            Op::Incrr { src } => {
                let rstr = reg_str(src);
                let line = format!("incrr {}", rstr);
                initial.push_str(&line);
            }
            Op::Decrr { src } => {
                let rstr = reg_str(src);
                let line = format!("decrr {}", rstr);
                initial.push_str(&line);
            }
            Op::Fn { name, params } => {
                let line = format!("fn @{} {:?}", name, params);
                initial.push_str(&line);
            }
            Op::FnRetR { src } => {
                let rstr = reg_str(src);
                let line = format!("retr {}", rstr);
                initial.push_str(&line);
            }
            Op::FnRet => {
                let line = format!("ret \t ");
                initial.push_str(&line);
            }
            Op::Call { name, .. } => {
                let line = format!("call {} ", name);
                initial.push_str(&line);
            }
            Op::FnStackPush { src } => {
                let rstr = reg_str(src);
                let line = format!("fstpush {} ", rstr);
                initial.push_str(&line);
            }
            Op::FnStackPop { dest } => {
                let rstr = reg_str(dest);
                let line = format!("fstpop {}", rstr);
                initial.push_str(&line);
            }
            Op::TblI { tabname, key, src } => {
                let rstr = reg_str(src);
                let line = format!("tbli {} {} {}", rstr, key, tabname);
                initial.push_str(&line);
            }
            Op::TblG { tabname, key, dest } => {
                let rstr = reg_str(dest);
                let line = format!("tblg {} {} {}", tabname, key, rstr);
                initial.push_str(&line);
            }
            Op::Stop => {
                let line = format!("{}\t", "stop");
                initial.push_str(&line);
            }
        };

        write!(f, "{}", initial)
    }
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
        SifVal::Tab(t) => format!("{:?}", t),
    }
}
