use crate::{
    block::SifBlockRef,
    cfg::CFG,
    ssa::phi::{PhiFn, PhiOp},
};
use sifc_bytecode::{instr::Instr, opc::Op};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

pub struct SSABuilder {
    /// Current control flow graph. The builder operates on this graph and transforms it into
    /// SSA form. This involves replacing and overwriting several components within the graph.
    pub cfg: CFG,

    /// Set of global variables, populated by the get_globs() method. Globals contains
    /// names that are used in a block that is separate from its definition, not specifically
    /// a global variable.
    globs: HashSet<String>,

    /// A mapping of variable names to blocks that contain that name. This is used mostly for
    /// getting the list of blocks we may need to insert phi functions into.
    blks: HashMap<String, Vec<SifBlockRef>>,

    /// "Rewrite counter": This is a map from variable name to usage count. We use this to get
    /// the current usage value for rewriting variable names.
    rwcounter: HashMap<String, usize>,

    /// A map of stacks for each global variable name. We grab the top value from the stack when
    /// rewriting var names. The values are inserted by newname(), which uses the rwcounter to
    /// store the usage counts.
    rwstack: HashMap<String, Vec<usize>>,
}

struct RWInstrs {
    /// Vector of rewritten instructions with new variable names
    pub instrs: Vec<Instr>,

    /// Amount of instructions that were changed during rw_instrs phase
    pub changed: usize,
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
        let mut prev_usages = HashSet::new();

        for block in &self.cfg.nodes {
            let mut varkill: HashSet<String> = HashSet::new();

            for i in &block.borrow().instrs {
                // we only care about instructions that set or load variables. Loading or setting
                // registers doesn't matter for phi function insertion.
                match i.op.clone() {
                    Op::Ldn { dest: _, name } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name);
                    }
                    Op::Ldas { name, .. } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name);
                    }
                    Op::Ldav { name, .. } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name);
                    }
                    Op::Upda { name, .. } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name);
                    }
                    Op::Stn { srcname, destname } => {
                        if !varkill.contains(&srcname) && prev_usages.contains(&srcname) {
                            self.globs.insert(srcname.clone());
                        }
                        prev_usages.insert(srcname);

                        if !varkill.contains(&destname) && prev_usages.contains(&destname) {
                            self.globs.insert(destname.clone());
                        }
                        prev_usages.insert(destname.clone());

                        varkill.insert(destname.clone());
                        let mut curr = self.blks.get(&destname).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(destname, curr);
                    }
                    Op::Stc { val: _, name } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name.clone());
                        varkill.insert(name.clone());

                        let mut curr = self.blks.get(&name).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(name, curr);
                    }
                    Op::Str { src: _, name } => {
                        if !varkill.contains(&name) && prev_usages.contains(&name) {
                            self.globs.insert(name.clone());
                        }
                        prev_usages.insert(name.clone());
                        varkill.insert(name.clone());

                        let mut curr = self.blks.get(&name).cloned().unwrap_or(Vec::new());
                        curr.push(Rc::clone(block));
                        self.blks.insert(name, curr);
                    }
                    Op::Tbli { tabname, .. } => {
                        if !varkill.contains(&tabname) && prev_usages.contains(&tabname) {
                            self.globs.insert(tabname.clone());
                        }
                        prev_usages.insert(tabname);
                    }
                    Op::Tblg { tabname, .. } => {
                        if !varkill.contains(&tabname) && prev_usages.contains(&tabname) {
                            self.globs.insert(tabname.clone());
                        }
                        prev_usages.insert(tabname);
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
            // Build a queue of blocks to iterate over when inserting phis. This allows
            // us to pull from the queue and also add to the end as we iterate.
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
                        // Insert new phi function for name. Operands are currently copies
                        // of the var name, which will be overwritten later during the renaming
                        // phase. The amount of operands is equal to the amount of possible previous
                        // usages of the destination. This is usually two, but not necessarily always.
                        // We don't really need to set operands here anyway, since they should be
                        // overwritten by renaming as well.
                        let pops = Vec::new();
                        let phi = PhiFn::new(name.to_string(), name.to_string(), pops);
                        d.borrow_mut().phis.insert(name.to_string(), phi);
                        queue.push_back(Rc::clone(&d));
                    }
                }
            }
        }
    }

    fn rewrite(&mut self) {
        for name in &self.globs {
            self.rwcounter.insert(name.to_string(), 1);
            self.rwstack.insert(name.to_string(), vec![1]);
        }
        self.rename_block(Rc::clone(&self.cfg.graph));
    }

    fn rename_block(&mut self, block: SifBlockRef) {
        // Rename phi function dests
        let mut rw_phis = HashMap::new();
        for (on, phi) in &block.borrow().phis {
            let n = self.newname(&phi.dest);
            let newphi = PhiFn::new(on.clone(), n.clone(), phi.operands.clone());
            rw_phis.insert(n, newphi);
        }
        block.borrow_mut().phis = rw_phis;

        // Rename instructions
        let rwinsts = self.rw_instrs(block.borrow().instrs.clone());
        block.borrow_mut().instrs = rwinsts.instrs.clone();

        // Recursively rename each immediate successor in the dom tree. dom_tree_node
        // is cloned, but as long as we recursively call with a reference
        // to the actual node being renamed that's fine.
        let dom_tree_node = &self.cfg.dom_tree.nodes[block.borrow().id].clone();
        for bid in &dom_tree_node.edges {
            let next = Rc::clone(&self.cfg.nodes[*bid]);
            self.rename_block(next);
        }

        // Rename phi function params in immediate cfg successors.
        // We only write a single operand here, in each phi in the succesors. Each block should only
        // write to a specific operand "slot" when rewriting, which is defined by the block id. This slot
        // doesn't necessarily need to correspond to the amount of operands, but it is always the
        // same for that block. If we need the operands in a list ordered by slot, we can perform that
        // transformation later.
        for cfg_succ in &block.borrow().edges {
            let mut rw_phis_ops = HashMap::new();
            for (name, phi) in &cfg_succ.borrow_mut().phis {
                let mut new_os = phi.operands.clone();
                let initial_dest = &phi.initial;

                // If the initial name (ie. the destination) is in our stacks,
                // add an operand for the rewritten name.
                if let Some(subscript) = self.rwstack.get(initial_dest) {
                    let nn = format!("{}{}", initial_dest, self.top(subscript));
                    let new_op = PhiOp::new(block.borrow().id, nn);
                    new_os.push(new_op);
                }

                // Clone the previoud phi but with additional operand.
                let newphi = PhiFn::new(name.clone(), phi.dest.clone(), new_os);
                rw_phis_ops.insert(name.to_string(), newphi);
            }
            cfg_succ.borrow_mut().phis = rw_phis_ops;
        }

        // Pop subscripts from rwstack for dest names in phis and instrs
        for (_on, phi) in &block.borrow().phis {
            self.pop_discard(&phi.dest);
        }
        self.pop_remaining(rwinsts.instrs);
    }

    fn rw_instrs(&mut self, instrs: Vec<Instr>) -> RWInstrs {
        let mut newinsts = Vec::new();
        let mut changed = 0;

        for i in &instrs {
            // we only care about instructions that set or load variables. Loading or setting
            // registers doesn't matter for phi function insertion.
            match i.op.clone() {
                Op::Ldn { dest, name } => {
                    if let Some(subscript_stack) = self.rwstack.get(&name) {
                        let nn = format!("{}{}", name, self.top(subscript_stack));

                        let new_op = Op::Ldn {
                            dest: dest,
                            name: nn,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Ldas { name, dest } => {
                    if let Some(subscript_stack) = self.rwstack.get(&name) {
                        let nn = format!("{}{}", name, self.top(subscript_stack));

                        let new_op = Op::Ldas {
                            name: nn,
                            dest: dest,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Ldav {
                    name,
                    idx_reg,
                    dest,
                } => {
                    if let Some(subscript_stack) = self.rwstack.get(&name) {
                        let nn = format!("{}{}", name, self.top(subscript_stack));

                        let new_op = Op::Ldav {
                            name: nn,
                            idx_reg: idx_reg,
                            dest: dest,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Upda {
                    name,
                    idx_reg,
                    val_reg,
                } => {
                    if let Some(subscript_stack) = self.rwstack.get(&name) {
                        let nn = format!("{}{}", name, self.top(subscript_stack));

                        let new_op = Op::Upda {
                            name: nn,
                            idx_reg: idx_reg,
                            val_reg: val_reg,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Stn { srcname, destname } => {
                    if let Some(srcsubscript) = self.rwstack.get(&srcname) {
                        let nsrc = format!("{}{}", srcname, self.top(srcsubscript));
                        let ndest = self.newname(&destname);
                        changed += 1;

                        let new_op = Op::Stn {
                            srcname: nsrc,
                            destname: ndest,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Stc { val, name } => {
                    let nn = self.newname(&name);
                    changed += 1;

                    let new_op = Op::Stc { val: val, name: nn };
                    let new_inst = Instr::new(i.lblidx, new_op, i.line);
                    newinsts.push(new_inst);
                }
                Op::Str { src, name } => {
                    let nn = self.newname(&name);
                    changed += 1;

                    let new_op = Op::Str { src: src, name: nn };
                    let new_inst = Instr::new(i.lblidx, new_op, i.line);
                    newinsts.push(new_inst);
                }
                Op::Tbli { tabname, key, src } => {
                    if let Some(subscript_stack) = self.rwstack.get(&tabname) {
                        let nn = format!("{}{}", tabname, self.top(subscript_stack));

                        let new_op = Op::Tbli {
                            tabname: nn,
                            key: key,
                            src: src,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                Op::Tblg { tabname, key, dest } => {
                    if let Some(subscript_stack) = self.rwstack.get(&tabname) {
                        let nn = format!("{}{}", tabname, self.top(subscript_stack));

                        let new_op = Op::Tblg {
                            tabname: nn,
                            key: key,
                            dest: dest,
                        };
                        let new_inst = Instr::new(i.lblidx, new_op, i.line);
                        newinsts.push(new_inst);
                    }
                }
                _ => newinsts.push(i.clone()),
            }
        }

        RWInstrs {
            instrs: newinsts,
            changed: changed,
        }
    }

    // Anytime we call newname in rw_instrs, we need to pop from the appropriate stack here.
    fn pop_remaining(&mut self, instrs: Vec<Instr>) {
        for i in &instrs {
            match i.op.clone() {
                Op::Stn {
                    srcname: _,
                    destname,
                } => self.pop_discard(&destname),
                Op::Stc { val: _, name } => self.pop_discard(&name),
                Op::Str { src: _, name } => self.pop_discard(&name),
                _ => {}
            }
        }
    }

    fn newname(&mut self, old: &str) -> String {
        let mb_i = self.rwcounter.get(old).cloned();
        if mb_i.is_none() {
            return format!("{}{}", old, 0);
        }

        let i = mb_i.unwrap();
        self.rwcounter.insert(old.to_string(), i + 1);
        let mut stk = self.rwstack.get(old).cloned().unwrap();
        stk.push(i);
        self.rwstack.insert(old.to_string(), stk);

        format!("{}{}", old, i)
    }

    fn top(&self, stack: &Vec<usize>) -> usize {
        stack[stack.len() - 1]
    }

    fn pop_discard(&mut self, st_name: &str) {
        if let Some(st) = self.rwstack.get(st_name) {
            let mut ns = st.clone();
            ns.pop();
            self.rwstack.insert(st_name.to_string(), ns);
        }
    }
}
