use crate::sifv::SifVal;

/// DReg represents a data register.
#[derive(Clone, Debug)]
pub struct DReg {
    pub name: String,
    pub cont: Option<SifVal>,
}

impl DReg {
    pub fn new(n: String) -> DReg {
        DReg {
            name: n,
            cont: None,
        }
    }

    pub fn from_cont(n: String, c: Option<SifVal>) -> DReg {
        DReg { name: n, cont: c }
    }

    pub fn is_empty(&self) -> bool {
        self.cont.is_none()
    }
}
