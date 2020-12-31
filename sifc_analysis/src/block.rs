use crate::ssa::phi::PhiFn;
use sifc_bytecode::instr::Instr;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
};

// See the following for rust graph representation explanations:
// http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
// https://github.com/nrc/r4cppp/blob/master/graphs/src/rc_graph.rs
pub(crate) type SifBlockRef = Rc<RefCell<SifBlock>>;
pub(crate) type BlockID = usize;

/// Represents something like a basic block. This is just a standard graph vertex implementation,
/// but the data it holds is a list of instructions in the block.
#[derive(Clone, PartialEq)]
pub struct SifBlock {
    /// String identifier.
    pub name: String,

    /// Usize identifier.
    pub id: BlockID,

    /// Sif IR instruction vec. This should be the full program that would
    /// normally be executed.
    pub instrs: Vec<Instr>,

    /// Adjacent blocks.
    pub edges: Vec<SifBlockRef>,

    /// Predecessor blocks. This includes all blocks that can be reached when
    /// traversing to the current block.
    pub preds: Vec<SifBlockRef>,

    /// Set of dominators. We can build a list of SifBlockRefs from this set
    /// if needed, but we mostly need to check BlockID's for dominators.
    pub dom_set: HashSet<BlockID>,

    /// Immediate dominator block id. If the block is the entry node to a CFG, the idom
    /// is None.
    pub idom: Option<BlockID>,

    /// Dominance frontier. This contains "the first nodes reachable from n that n does not
    /// dominate, on each path leaving n"
    pub dom_front: HashSet<BlockID>,

    /// Set of phi functions on this block. The keys are the names of variables appearing in
    /// predecessors of this block, which require the phi function to be inserted.
    pub phis: HashMap<String, PhiFn>,
}

impl SifBlock {
    pub fn new(name: &str, id: usize) -> SifBlockRef {
        let block = SifBlock {
            name: name.to_string(),
            id: id,
            instrs: Vec::new(),
            edges: Vec::new(),
            preds: Vec::new(),
            dom_set: HashSet::new(),
            idom: None,
            dom_front: HashSet::new(),
            phis: HashMap::new(),
        };

        Rc::new(RefCell::new(block))
    }

    pub fn add_instr(&mut self, i: &Instr) {
        self.instrs.push(i.clone());
    }
}

// Impl debug for blocks so we don't accidentally follow edges and potentially end up in
// an infinite loop if the CFG has a cycle -_-
impl fmt::Debug for SifBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bl = String::new();
        bl.push_str(&format!("Block {}: {{ \n", self.id));

        let mut edge_vec = Vec::new();
        for edge in &self.edges {
            edge_vec.push(edge.borrow().id);
        }

        bl.push_str(&format!("Edge BIDs: {:#?}\n", edge_vec));

        let mut pred_vec = Vec::new();
        for edge in &self.preds {
            pred_vec.push(edge.borrow().id);
        }

        bl.push_str(&format!("Pred BIDs: {:#?}\n", pred_vec));
        bl.push_str(&format!("DOM Set BIDs: {:#?}\n", self.dom_set));
        bl.push_str(&format!("IDOM: {:#?}\n", self.idom));
        bl.push_str(&format!("DOM Frontier BIDs: {:#?}\n", self.dom_front));
        bl.push_str(&format!("Phis: {:#?}\n", self.phis));

        bl.push_str(&format!("Instrs:\n"));
        for i in &self.instrs {
            // better to use display and not debug here, but technically both work
            bl.push_str(&format!("{:#}\n", i));
        }

        bl.push_str("}");
        write!(f, "{}", bl)
    }
}
