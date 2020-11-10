use sifc_bytecode::{instr::Instr, opc::Op};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

// There is almost certainly a more elegant way to express a graph than this, but a list of
// nodes, each containing a lsit of edges should probably suffice. To walk the graph, we can
// use a standard iterative DFS/BFS and use the visited field in each block to indicate when
// a block has been processed. The downside to this is that if we want to traverse a CFG
// multiple times we need to reset that field, I think?
// For a perhaps better example, see:
// http://smallcultfollowing.com/babysteps/blog/2015/04/06/modeling-graphs-in-rust-using-vector-indices/
pub type CFG = Vec<SifBlock>;
type BlockID = usize;

#[derive(Debug, Clone)]
pub struct SifBlock {
    pub name: String,
    pub id: BlockID,
    pub instrs: Vec<Instr>,
    pub edges: Vec<Rc<RefCell<SifBlock>>>,
}

impl SifBlock {
    pub fn new(name: &str, id: usize) -> Rc<RefCell<SifBlock>> {
        let block = SifBlock {
            name: name.to_string(),
            id: id,
            instrs: Vec::new(),
            edges: Vec::new(),
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
// Overall we still run in O(n) time and O(m) space, where n is the instruction count
// and m is the number of blocks.
// TODO: do we build an inter procedural cfg or treat each method as a separate cfg?
// TODO: Do function calls split blocks from code section to decl section?
pub fn build_cfg(instrs: Vec<Instr>) -> Rc<RefCell<SifBlock>> {
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

    entry_block
}
