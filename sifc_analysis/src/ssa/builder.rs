use crate::{block::SifBlockRef, cfg::CFG, ssa::phi::PhiFn};
use sifc_bytecode::opc::Op;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

pub struct SSABuilder {
    cfg: CFG,
    globs: HashSet<String>,
    blks: HashMap<String, Vec<SifBlockRef>>,
}

impl SSABuilder {
    pub fn new(cfg: CFG) -> SSABuilder {
        SSABuilder {
            cfg: cfg,
            globs: HashSet::new(),
            blks: HashMap::new(),
        }
    }

    pub fn build(&mut self) {
        self.get_globs();
        self.insert_phis();
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
    /// insert_globs() has already been called and the resulting sets are filled.
    fn insert_phis(&mut self) {
        for name in &self.globs {
            let list = self.blks.get(name).cloned().unwrap_or(Vec::new());
            let mut queue = VecDeque::new();
            for bref in list {
                queue.push_front(Rc::clone(&bref));
            }

            while queue.len() != 0 {
                let curr = queue.pop_front().unwrap();
                for bid in &curr.borrow().dom_front {
                    let d = &self.cfg.nodes[*bid];
                    if !d.borrow().phis.contains_key(name) {
                        // Insert new phi function for name
                        // TODO: Operands?
                        let phi = PhiFn::new();
                        d.borrow_mut().phis.insert(name.to_string(), phi);
                        queue.push_back(Rc::clone(&d));
                    }
                }
            }
        }
    }
}
