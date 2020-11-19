use crate::{
    cfg::{SifBlockRef, CFG},
    ssa::SSAVal,
};
use std::collections::{HashMap, HashSet};

pub struct ValueNumbering {
    cfg: CFG,
    curr_defs: HashMap<String, SSAVal>,
    var_count: usize,
    sealed_blocks: HashSet<usize>,
}

impl ValueNumbering {
    pub fn new(cfg: CFG) -> ValueNumbering {
        ValueNumbering {
            cfg: cfg,
            curr_defs: HashMap::new(),
            var_count: 0,
            sealed_blocks: HashSet::new(),
        }
    }

    /// Performs local value numbering for a block of instructions
    fn lvn(&self) {}

    fn write_var(&mut self, var: usize, block: SifBlockRef, rhs: SSAVal) {
        let block_id = block.borrow().id;
        let key = self.encode(var, block_id);
        self.curr_defs.insert(key, rhs);
        self.var_count += 1;
    }

    fn read_var(&mut self, var: usize, block: SifBlockRef) -> SSAVal {
        let block_id = block.borrow().id;
        let key = self.encode(var, block_id);
        if self.curr_defs.contains_key(&key) {
            let val = self.curr_defs.get(&key).unwrap().clone();
            return val;
        }

        self.read_var_gvn(var, block)
    }

    fn read_var_gvn(&mut self, var: usize, block: SifBlockRef) -> SSAVal {
        let mut val = SSAVal::Empty;
        let block_id = block.borrow().id;
        if !self.sealed_blocks.contains(&block_id) {
            // 1. make new phi
            // 2. store phi in incomplete phis set
        } else if block.borrow().preds.len() == 1 {
            val = self.read_var(var, block);
            return val;
        } else {
            // 1. make phi
            // 2. write phi val
            // 3. put operands into phi
        }
        // 1. write val into var
        // 2. return val
        // self.write_var(var, block, val);
        val
    }

    fn encode(&self, var: usize, block_id: usize) -> String {
        format!("{}:{}", var, block_id)
    }

    fn decode(&self, key: String) -> (String, usize) {
        let parts: Vec<&str> = key.split(":").collect();
        (parts[0].parse().unwrap(), parts[1].parse().unwrap())
    }
}
