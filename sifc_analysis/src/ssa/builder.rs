use crate::{block::SifBlockRef, cfg::CFG, ssa::phi::PhiFn};
use sifc_bytecode::opc::Op;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

pub struct SSABuilder {
    pub cfg: CFG,

    globs: HashSet<String>,
    blks: HashMap<String, Vec<SifBlockRef>>,
    rwcounter: HashMap<String, usize>,
    rwstack: HashMap<String, Vec<usize>>,
}

impl SSABuilder {
    pub fn new(cfg: CFG) -> SSABuilder {
        SSABuilder {
            cfg: cfg,
            globs: HashSet::new(),
            blks: HashMap::new(),
            rwcounter: HashMap::new(),
            rwstack: HashMap::new(),
        }
    }

    /// Translates the instructions in the CFG into SSA form. Follows the procedure:
    /// 1. Determine variable names that appear in blocks but are defined in other blocks. These
    ///    are referred to as "globals".
    /// 2. Insert phi functions into blocks that use variables determined as globals. Phis are
    ///    stored in SifBlock.phis, but can be considered to be at the "head" of the block.
    /// 3. Rewrite variable names in each block to ensure there is only 1 occurrence of each, including
    ///    in phi function operands.
    /// This overwrites the blocks in the given CFG rather than returning a copy. After required analysis
    /// is done on SSA form, it can be translated back into regular SifIR form before execution or
    /// further translation.
    pub(crate) fn build(&mut self) {
        self.get_globs();
        self.insert_phis();
        self.rewrite();
    }

    /// Determine "global" names. Global refers to names of variables that are defined outside
    /// of the current block (determined by finding their usages in other blocks). We use these
    /// globals to determine which variables need phi functions, not necessarily where the phi
    /// functions are inserted.
    /// This method also fills self.blks, which contains a mapping of names to the blocks that
    /// contain definitions of the name.
    fn get_globs(&mut self) {
        for block in &self.cfg.nodes {
            let mut varkill: HashSet<String> = HashSet::new();

            for i in &block.borrow().instrs {
                // we only care about instructions that set or load variables. Loading or setting
                // registers doesn't matter for phi function insertion.
                match i.op.clone() {
                    Op::Ldn { dest: _, name } => {
                        if !varkill.contains(&name) {
                            self.globs.insert(name);
                        }
                    }
                    Op::Ldas { name, .. } => {
                        if !varkill.contains(&name) {
                            self.globs.insert(name);
                        }
                    }
                    Op::Ldav { name, .. } => {
                        if !varkill.contains(&name) {
                            self.globs.insert(name);
                        }
                    }
                    Op::Upda { name, .. } => {
                        if !varkill.contains(&name) {
                            self.globs.insert(name);
                        }
                    }
                    Op::Stn { srcname, destname } => {
                        if !varkill.contains(&srcname) {
                            self.globs.insert(srcname);
                        }

                        varkill.insert(destname.clone());
                        let mut curr = self.blks.get(&destname).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(destname, curr);
                    }
                    Op::Stc { val: _, name } => {
                        varkill.insert(name.clone());
                        let mut curr = self.blks.get(&name).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(name, curr);
                    }
                    Op::Str { src: _, name } => {
                        varkill.insert(name.clone());
                        let mut curr = self.blks.get(&name).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(name, curr);
                    }
                    Op::Tbli { tabname, .. } => {
                        if !varkill.contains(&tabname) {
                            self.globs.insert(tabname);
                        }
                    }
                    Op::Tblg { tabname, .. } => {
                        if !varkill.contains(&tabname) {
                            self.globs.insert(tabname);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Push phi functions to the head of each block where required. This assumes that
    /// get_globs() has already been called and the resulting sets are filled.
    /// From Engineering a Compiler pp.501
    fn insert_phis(&mut self) {
        for name in &self.globs {
            // Build a queue of blocks to iterate over when inserting phis
            let list = self.blks.get(name).cloned().unwrap_or(Vec::new());
            let mut queue = VecDeque::new();
            for bref in list {
                queue.push_front(Rc::clone(&bref));
            }

            while queue.len() != 0 {
                let curr = queue.pop_front().unwrap();

                // For each block in the dominance frontier, if that block does
                // not already contain a phi function for the current name, create one
                // and insert it.
                for bid in &curr.borrow().dom_front {
                    // Requires the cfg nodes array to be in order to match the ids. This should be
                    // correct if using the cfg construction in this crate.
                    let d = &self.cfg.nodes[*bid];

                    if !d.borrow().phis.contains_key(name) {
                        // Insert new phi function for name. Operands are just two copies of
                        // the current name, they have to be rewritten later anyway.
                        let pops = vec![name.to_string(), name.to_string()];
                        let phi = PhiFn::new(name.to_string(), pops);
                        d.borrow_mut().phis.insert(name.to_string(), phi);
                        queue.push_back(Rc::clone(&d));
                    }
                }
            }
        }
    }

    fn rewrite(&mut self) {
        self.rename_block();
    }

    fn rename_block(&mut self) {}

    fn newname(&mut self, old: &str) -> String {
        let i = self.rwcounter.get(old).cloned().unwrap();
        self.rwcounter.insert(old.to_string(), i + 1);
        let mut stk = self.rwstack.get(old).cloned().unwrap();
        stk.push(i);
        self.rwstack.insert(old.to_string(), stk);
        format!("{}{}", old, i)
    }
}
