use sifc_bytecode::{instr::Instr, opc::Op};
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

// See the following for rust graph representation explanations:
// http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
// https://github.com/nrc/r4cppp/blob/master/graphs/src/rc_graph.rs
pub type SifBlockRef = Rc<RefCell<SifBlock>>;
type BlockID = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct CFG {
    pub num_nodes: usize,
    pub graph: SifBlockRef,
}

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

// Builds a complete CFG from a list of instructions. This roughly follows the
// algorithm outlined in Engineering a Compiler 2nd Ed., p.241, but we have a
// favorable structure for cfg building so this algorithm is a little simpler.
// We don't calculate and store a list of leaders here, because we just compare
// instruction labels to determine if we have a potential leader. We don't end up saving
// any time from this though, because we still need an initial pass to initialize block
// structures. The second pass is very similar to a textbook algorithm, which just
// checks the instruction type and inserts edges for jumps.
// Overall we still run in O(n) time.
// TODO: do we build an inter procedural cfg or treat each method as a separate cfg?
// TODO: Do function calls split blocks from code section to decl section?
pub fn build_cfg(instrs: &Vec<Instr>) -> CFG {
    if instrs.len() == 0 {
        Rc::new(RefCell::new(SifBlock::new("entry", usize::MAX)));
    }

    // First, we initialize our blocks and add instructions to them. The first
    // loop ensures that we have block structs in memory with correct instructions
    // so that we can decide how to make edges to/from them in a later pass.
    let entry_block = SifBlock::new(&instrs[0].lbl, instrs[0].lblidx);

    let mut nodes = Vec::new();
    nodes.push(Rc::clone(&entry_block));

    let mut i = 1;
    let mut curr_idx = 0;
    nodes[curr_idx].borrow_mut().add_instr(&instrs[0]);

    while i < instrs.len() {
        let curr = &instrs[i];
        let prev = &instrs[i - 1];

        // If the current label is not the same as the previous one, this instruction
        // is a block leader. We make a new block for it and add it to the CFG.
        if curr.lblidx != prev.lblidx {
            let new_block = SifBlock::new(&curr.lbl, curr.lblidx);
            nodes.push(Rc::clone(&new_block));
            curr_idx += 1;
        }

        nodes[curr_idx].borrow_mut().add_instr(curr);
        i += 1;
    }

    // In the second loop, we add outgoing edges from each node. While not exactly specified,
    // because each edge is just represented by a usize (an index into the list of blocks
    // that is the CFG), edges are directed.
    // Edges are added when jmps are found, and similarly when we have "implicit" jumps. That is,
    // when an instruction is a block leader but the previous block did not end on a jump. We could
    // insert jumps at the end of every block to simplify this code a bit and make it more explicit,
    // but that would also make the compiler even more "block-aware" which may not be the best for
    // maintainability.
    i = 1;
    while i < instrs.len() {
        let curr = &instrs[i];
        let prev = &instrs[i - 1];

        match curr.op {
            Op::JmpCnd {
                kind: _,
                src: _,
                lblidx,
            } => {
                // For conditional jumps, we need to add the potential jump target AND the
                // subsequent block in case the condition is false and we fall through to the
                // following block.
                let mut curr_block = nodes[curr.lblidx].borrow_mut();
                curr_block.edges.push(Rc::clone(&nodes[lblidx]));
                curr_block.edges.push(Rc::clone(&nodes[lblidx - 1]));
            }
            Op::Jmpa { lblidx } => {
                let curr_block = &mut nodes[curr.lblidx].borrow_mut();
                curr_block.edges.push(Rc::clone(&nodes[lblidx]));
            }
            _ => {
                if curr.lblidx != prev.lblidx {
                    match prev.op {
                        Op::JmpCnd { .. } | Op::Jmpa { .. } | Op::FnRet => {}
                        _ => {
                            let curr_block = &mut nodes[prev.lblidx].borrow_mut();
                            curr_block.edges.push(Rc::clone(&nodes[curr.lblidx]));
                        }
                    };
                }
            }
        };
        i += 1;
    }

    build_preds(&nodes, Rc::clone(&entry_block));
    dom_calc(&nodes);

    CFG {
        num_nodes: nodes.len(),
        graph: entry_block,
    }
}

/// Add predecessors to nodes in the cfg. This information is primarly used for SSA construction.
/// This treats predecessors as any node that is visited on a path to the current node, and thus
/// contains a list of nodes rather than just the direct predecessor. Because we use a HashSet to
/// store nodes here, the order is not guaranteed and thus we cannot determine the direct predecessor
/// from this list at a later point.
fn build_preds(nodes: &Vec<SifBlockRef>, entry: SifBlockRef) {
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_front(Rc::clone(&entry));
    seen.insert(entry.borrow().id);

    while queue.len() != 0 {
        let curr = queue.pop_front().unwrap();

        for adj in &curr.borrow().edges {
            if !seen.contains(&adj.borrow().id) {
                for pred_id in &seen {
                    let pred = Rc::clone(&nodes[*pred_id]);
                    adj.borrow_mut().preds.push(pred);
                }
                seen.insert(adj.borrow().id);
                queue.push_back(Rc::clone(&adj));
            }
        }
    }
}

fn dom_calc(nodes: &Vec<SifBlockRef>) {
    nodes[0].borrow_mut().dom = vec![Rc::clone(&nodes[0])];
    nodes[0].borrow_mut().dom_set = [0].iter().cloned().collect();

    let mut full_dom_set = HashSet::new();
    for node in nodes {
        full_dom_set.insert(node.borrow().id);
    }

    let mut i = 1;
    while i < nodes.len() {
        nodes[i].borrow_mut().dom = nodes.clone();
        nodes[i].borrow_mut().dom_set = full_dom_set.clone();
        i += 1;
    }

    let mut changed = true;
    while changed {
        changed = false;

        let mut i = 1;
        while i < nodes.len() {
            let node = &nodes[i];

            let mut pred_dom_intersection = dom_intersection(&node.borrow().preds);
            pred_dom_intersection.insert(0, Rc::clone(&node));

            if pred_dom_intersection != node.borrow().dom {
                node.borrow_mut().dom = pred_dom_intersection;
                changed = true;
            }

            i += 1;
        }
    }
}

fn dom_intersection(preds: &Vec<SifBlockRef>) -> Vec<SifBlockRef> {
    let mut sets = Vec::new();
    let mut i = 1;
    while i < preds.len() {
        let pred = &preds[i];
        let dom_set = pred.borrow().dom_set.clone();
        sets.push(dom_set);
        i += 1;
    }

    let initial = preds[0].borrow().dom_set.clone();
    let intersection: HashSet<BlockID> = initial
        .iter()
        .filter(|k| sets.iter().all(|s| s.contains(k)))
        .cloned()
        .collect();

    let mut result = Vec::new();
    for pred in preds {
        let id = pred.borrow().id;
        if intersection.contains(&id) {
            result.push(Rc::clone(&pred));
        }
    }

    result
}
