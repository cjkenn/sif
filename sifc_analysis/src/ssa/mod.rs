mod phi;
mod val_num;

use phi::PhiFn;
use sifc_bytecode::sifv::SifVal;

#[derive(Debug, Clone)]
pub enum SSAVal {
    Val(SifVal),
    Phi(PhiFn),
    Empty,
}
