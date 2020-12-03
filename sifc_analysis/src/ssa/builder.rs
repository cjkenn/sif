use crate::{
    cfg::{SifBlockRef, CFG},
    ssa::{PhiFn, SSAVal},
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct SSABuilder {
    cfg: CFG,
    curr_defs: HashMap<String, SSAVal>,
    var_count: usize,
    sealed_blocks: HashSet<usize>,
    incomplete_phis: HashMap<String, SSAVal>,
}

impl SSABuilder {
    pub fn new(cfg: CFG) -> SSABuilder {
        SSABuilder {
            cfg: cfg,
            curr_defs: HashMap::new(),
            var_count: 0,
            sealed_blocks: HashSet::new(),
            incomplete_phis: HashMap::new(),
        }
    }

    pub fn build(&self) {
        // Input: CFG with instrs
        // Ouput: CFG with SASInsts? What type do we use?
        //
        // 1. Need to convert SifVals to SSAVals? This can probably be done while
        //    traversal is happening?
        // 2. Traverse the cfg. At each block, we must iterate each instruction
        //    contained in the block.
        // 3. Upon variable declarations/assignments, we call write_var. This correspond to
        //    stc and stn instructions.
        // 4. Upon any variable reads, we call read_var. What do we do with the result?
        //    These correspond to ldn, ldc instructions
    }

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
            let phi = PhiFn::new(Rc::clone(&block));
            val = SSAVal::Phi(phi);
            // 2. store phi in incomplete phis set
            self.insert_incomplete_phi(block_id, var, val.clone());
        } else if block.borrow().preds.len() == 1 {
            val = self.read_var(var, Rc::clone(&block));
        } else {
            // 1. make phi
            let phi = PhiFn::new(Rc::clone(&block));
            let phi_val = SSAVal::Phi(phi.clone());
            self.write_var(var, Rc::clone(&block), phi_val.clone());
            // 2. write phi val
            // 3. put operands into phi
            val = self.add_phi_operands(var, phi_val);
        }
        // 1. write val into var
        // 2. return val
        self.write_var(var, block, val.clone());
        val
    }

    fn add_phi_operands(&mut self, var: usize, phi: SSAVal) -> SSAVal {
        match phi {
            SSAVal::Phi(phi) => {
                let mut new_phi = phi.clone();

                for pred in &phi.block.borrow().preds {
                    let op_val = self.read_var(var, Rc::clone(pred));
                    new_phi.operands.push(op_val);
                }

                return self.try_remove_trivial_phi(SSAVal::Phi(new_phi));
            }
            _ => panic!("ssa val is not a phi function!"),
        }
    }

    fn try_remove_trivial_phi(&mut self, phi_val: SSAVal) -> SSAVal {
        let mut same = SSAVal::Empty;
        match phi_val {
            SSAVal::Phi(ref phi) => {
                for op in &phi.operands {
                    if *op == same || *op == phi_val {
                        continue;
                    }

                    if same != SSAVal::Empty {
                        return phi_val;
                    }
                    same = op.clone();
                }

                if same == SSAVal::Empty {
                    same = SSAVal::Undef;
                }
            }
            _ => panic!("ssa val is not a phi function!"),
        }
        SSAVal::Empty
    }

    fn insert_curr_def(&mut self) {}

    fn insert_incomplete_phi(&mut self, block_id: usize, var: usize, val: SSAVal) {
        let key = format!("{}:{}", block_id, var);
        self.incomplete_phis.insert(key, val);
    }

    fn encode(&self, var: usize, block_id: usize) -> String {
        format!("{}:{}", var, block_id)
    }

    fn decode(&self, key: String) -> (String, usize) {
        let parts: Vec<&str> = key.split(":").collect();
        (parts[0].parse().unwrap(), parts[1].parse().unwrap())
    }
}
