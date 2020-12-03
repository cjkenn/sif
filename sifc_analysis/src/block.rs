use sifc_bytecode::instr::Instr;
use std::{cell::RefCell, collections::HashSet, rc::Rc};

// See the following for rust graph representation explanations:
// http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
// https://github.com/nrc/r4cppp/blob/master/graphs/src/rc_graph.rs
pub(crate) type SifBlockRef = Rc<RefCell<SifBlock>>;
pub(crate) type BlockID = usize;

/// Represents something like a basic block. This is just a standard graph vertex implementation,
/// but the data it holds is a list of instructions in the block.
#[derive(Debug, Clone, PartialEq)]
pub struct SifBlock {
    /// String identifier
    pub name: String,

    /// Usize identifier
    pub id: BlockID,

    /// Sif IR instruction vec. This should be the full program that would
    /// normally be executed
    pub instrs: Vec<Instr>,

    /// Adjacent blocks
    pub edges: Vec<SifBlockRef>,

    /// Predecessor blocks. This includes all blocks that can be reached when
    /// traversing to the current block
    pub preds: Vec<SifBlockRef>,

    /// List of dominators
    pub dom: Vec<SifBlockRef>,

    /// Set of dominators used for faster access to check dominators
    pub dom_set: HashSet<BlockID>,

    /// Immediate dominator block
    pub idom: Option<SifBlockRef>,
}

impl SifBlock {
    pub fn new(name: &str, id: usize) -> SifBlockRef {
        let block = SifBlock {
            name: name.to_string(),
            id: id,
            instrs: Vec::new(),
            edges: Vec::new(),
            preds: Vec::new(),
            dom: Vec::new(),
            dom_set: HashSet::new(),
            idom: None,
        };

        Rc::new(RefCell::new(block))
    }

    pub fn traverse<F>(&self, visit: &F, seen: &mut HashSet<BlockID>)
    where
        F: Fn(&Vec<Instr>),
    {
        if seen.contains(&self.id) {
            return;
        }

        visit(&self.instrs);
        seen.insert(self.id);

        for block in &self.edges {
            block.borrow().traverse(visit, seen);
        }
    }

    pub fn add_instr(&mut self, i: &Instr) {
        self.instrs.push(i.clone());
    }
}
