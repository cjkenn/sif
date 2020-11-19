mod phi;
mod val_num;

use phi::PhiFn;
use sifc_bytecode::sifv::SifVal;

#[derive(Debug, Clone, PartialEq)]
pub enum SSAVal {
    Val(SifVal),
    Phi(PhiFn),
    Empty,
    Undef,
}
