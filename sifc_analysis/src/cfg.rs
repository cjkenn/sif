use sifc_bytecode::{instr::Instr, opc::Op};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SifBlock {
    pub name: String,
    pub id: usize,
    pub instrs: Vec<Instr>,
    pub edges: Vec<SifBlock>,
    pub visited: bool,
}

impl SifBlock {
    pub fn new(name: &str, id: usize) -> SifBlock {
        SifBlock {
            name: name.to_string(),
            id: id,
            instrs: Vec::new(),
            edges: Vec::new(),
            visited: false,
        }
    }

    pub fn empty() -> SifBlock {
        SifBlock {
            name: "".to_string(),
            id: usize::MAX,
            instrs: Vec::new(),
            edges: Vec::new(),
            visited: false,
        }
    }
}

/// Builds a complete CFG from a list of instructions. This roughly follows the
/// algorithm outlined in Engineering a Compiler 2nd Ed., p.241, but we have a
/// favorable structure for cfg building so this algorithm is a little simpler.
// TODO: do we build an inter procedural cfg or treat each method as a separate cfg?
// TODO: Do function calls split blocks from code section to decl section?
pub fn build_cfg(instrs: Vec<Instr>) -> SifBlock {
    if instrs.len() == 0 {
        return SifBlock::new("entry", usize::MAX);
    }

    let lbl_count = instrs[instrs.len() - 1].lblidx + 1;

    let mut leaders = Vec::with_capacity(lbl_count);
    let entry_name = &instrs[0].lbl;
    let entry_id = instrs[0].lblidx;
    let entry_block = SifBlock::new(entry_name, entry_id);
    leaders.push(entry_block);

    // 1. Identify leaders by comparing labels: as we loop, if the labels change we need
    // to split the block. We must make an array or map of these blocks we create in the
    // first pass, so we can refer to them in the second pass and add edges. We should be able
    // to add instrs in the first pass. We can add edges from implicit jumps in this pass
    let mut i = 1;
    let mut curr_block_idx = 0;
    while i < instrs.len() {
        let instr = &instrs[i];

        if instr.lblidx != instrs[i - 1].lblidx {
            let new_block = SifBlock::new(&instr.lbl, instr.lblidx);
            leaders.push(new_block.clone());

            leaders[curr_block_idx].edges.push(new_block);
            curr_block_idx += 1;
        } else {
            leaders[curr_block_idx].instrs.push(instr.clone());
        }

        i += 1;
    }

    // 2. If there is a jump instr, we must add an edge. If there is a label change, there is an
    // implicit jump as control flow can fall through to the next label, so we must add an edge for
    // that case as well.
    for instr in instrs {
        match instr.op {
            Op::JmpCnd {
                kind: _,
                src: _,
                lblidx,
            } => {
                let dest_block = leaders[lblidx].clone();
                let curr_block = &mut leaders[instr.lblidx];
                curr_block.edges.push(dest_block);
            }
            Op::Jmpa { lblidx } => {
                let dest_block = leaders[lblidx].clone();
                let curr_block = &mut leaders[instr.lblidx];
                curr_block.edges.push(dest_block);
            }
            _ => {}
        };
    }

    let mut entry_block = SifBlock::new("entry", usize::MAX);
    entry_block.edges = leaders;
    entry_block
}
