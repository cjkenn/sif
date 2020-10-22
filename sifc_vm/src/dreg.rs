use sifc_compiler::sifv::SifVal;

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
}
