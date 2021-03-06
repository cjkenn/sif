use crate::{
    block::{SifBlock, SifBlockRef},
    dom,
};
use sifc_bytecode::{
    instr::Instr,
    opc::{JmpOpKind, Op},
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Debug, Clone)]
pub struct CFG {
    pub num_nodes: usize,
    /// Nodes contains each node in the CFG in a list in any order. This is useful for
    /// iterating over nodes to gather information when we do not need to traverse a graph.
    pub nodes: Vec<SifBlockRef>,

    /// The head of the CFG. Should normally be used for traversing the CFG.
    pub graph: SifBlockRef,

    /// Dominator tree of the CFG.
    pub dom_tree: dom::DomTree,
}

impl CFG {
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
    pub fn build(instrs: &Vec<Instr>) -> CFG {
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

            match &curr.op {
                Op::JmpCnd {
                    kind,
                    src: _,
                    lblidx,
                } => {
                    // For conditional jumps, we need to add the potential jump target AND the
                    // subsequent block in case the condition is false and we fall through to the
                    // following block.
                    let mut curr_block = nodes[curr.lblidx].borrow_mut();
                    curr_block.edges.push(Rc::clone(&nodes[*lblidx]));

                    let target_idx = match kind {
                        JmpOpKind::Jmpt => {
                            let tidx = curr.lblidx + 1;
                            if tidx >= nodes.len() {
                                None
                            } else {
                                Some(tidx)
                            }
                        }
                        JmpOpKind::Jmpf => Some(lblidx - 1),
                    };

                    if target_idx.is_some() {
                        curr_block
                            .edges
                            .push(Rc::clone(&nodes[target_idx.unwrap()]));
                    }
                }
                Op::Jmpa { lblidx } => {
                    let curr_block = &mut nodes[curr.lblidx].borrow_mut();
                    curr_block.edges.push(Rc::clone(&nodes[*lblidx]));
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

        // These must be performed in order. Preds first, then dominance information,
        // then dominance tree building.
        build_preds(&nodes, Rc::clone(&entry_block));
        dom::fill_doms(&nodes);
        let dtree = dom::DomTree::build(&nodes);

        CFG {
            num_nodes: nodes.len(),
            nodes: nodes,
            graph: entry_block,
            dom_tree: dtree,
        }
    }
}

/// Add predecessors to nodes in the cfg. This information is primarly used for SSA construction.
/// This treats predecessors as any node that is visited on a path to the current node, and thus
/// contains a list of nodes rather than just the direct predecessor. Because we use a HashSet to
/// store nodes here, the order is not guaranteed and thus we cannot determine the direct predecessor
/// from this list at a later point.
fn build_preds(nodes: &Vec<SifBlockRef>, entry: SifBlockRef) {
    let mut seen = HashSet::new();
    let mut stack = Vec::new();
    stack.push(Rc::clone(&entry));

    while stack.len() != 0 {
        let curr = stack.pop().unwrap();
        let curr_id = curr.borrow().id.clone();

        if !seen.contains(&curr_id) {
            for adj in &curr.borrow().edges {
                let pred = Rc::clone(&nodes[curr_id]);
                if !(adj.borrow().id == curr_id) {
                    adj.borrow_mut().preds.push(pred);
                    stack.push(Rc::clone(&adj));
                }
            }
            seen.insert(curr_id);
        }
    }
}
