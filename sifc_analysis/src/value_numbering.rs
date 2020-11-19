use crate::cfg::{SifBlock, CFG};
use sifc_bytecode::sifv::SifVal;
use std::collections::HashMap;

pub struct ValueNumbering {
    cfg: CFG,
    // TODO: need to use an SSAVal type instead of SifVal?
    curr_defs: HashMap<String, SifVal>,
}

impl ValueNumbering {
    pub fn new(cfg: CFG) -> ValueNumbering {
        ValueNumbering {
            cfg: cfg,
            curr_defs: HashMap::new(),
        }
    }

    /// Performs local value numbering for a block of instructions
    fn lvn(&self) {}

    fn write_var(&mut self, var: String, block_id: usize, rhs: SifVal) {
        let key = self.encode(&var, block_id);
        self.curr_defs.insert(key, rhs);
    }

    fn read_var(&self, var: String, block_id: usize) -> Option<&SifVal> {
        let key = self.encode(&var, block_id);
        if self.curr_defs.contains_key(&key) {
            return self.curr_defs.get(&key);
        }

        self.read_var_gvn(var, block_id)
    }

    fn read_var_gvn(&self, var: String, block_id: usize) -> Option<&SifVal> {
        None
    }

    fn encode(&self, var: &str, block_id: usize) -> String {
        format!("{}:{}", var, block_id)
    }

    fn decode(&self, key: String) -> (String, usize) {
        let parts: Vec<&str> = key.split(":").collect();
        (parts[0].to_string(), parts[1].parse().unwrap())
    }
}
