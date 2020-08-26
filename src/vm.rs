use crate::error::VMErr;
use crate::block::SifBlock;

pub struct VM {
    blocks: Vec<SifBlock>,
    ip: usize,
}

impl VM {
    pub fn new() -> VM {
        VM {
            blocks: Vec::new(),
            ip: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), VMErr> {
        unimplemented!()
    }
}
