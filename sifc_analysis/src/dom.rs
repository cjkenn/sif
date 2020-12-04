use crate::block::{BlockID, SifBlockRef};
use std::{collections::HashSet, rc::Rc};

/// Calculate dominance lists (and dominance sets) for each node in nodes.
/// This function will update each node in place with the correct dominance information.
/// Initial conditions:
/// 1. The entry node's dominator set includes only itself.
/// 2. For all other nodes (ie. not the entry), the dominator set
///    includes all nodes.
pub(crate) fn dom_calc(nodes: &Vec<SifBlockRef>) {
    // Set the dominators for the entry node, subject to initial conditions.
    nodes[0].borrow_mut().dom_set = [0].iter().cloned().collect();

    // Collect all node ids to build a set of initial dominators (all nodes),
    // for condition 2. above.
    let mut full_dom_set = HashSet::new();
    for node in nodes {
        full_dom_set.insert(node.borrow().id);
    }

    // Set the dominators for every node other than the entry node to be
    // all nodes.
    let mut i = 1;
    while i < nodes.len() {
        nodes[i].borrow_mut().dom_set = full_dom_set.clone();
        i += 1;
    }

    let mut changed = true;
    while changed {
        changed = false;

        let mut i = 1;
        while i < nodes.len() {
            let node = &nodes[i];

            // The temp dom set is the current node unioned with the intersection
            // of dominators in this node's predecessors.
            let mut pred_dom_intersection = dom_intersection(&node.borrow().preds);
            pred_dom_intersection.insert(node.borrow().id);

            if pred_dom_intersection != node.borrow().dom_set {
                node.borrow_mut().dom_set = pred_dom_intersection.clone();
                changed = true;
            }

            i += 1;
        }
    }
}

/// Calculates immediate dominators for each node in a CFG.
/// We  iterate over the node's dominator set and find the
/// item with the ID that is closest to the current block ID
/// (ie. the max block ID in the dominator set).
pub(crate) fn idom_calc(nodes: &Vec<SifBlockRef>) {
    for node in nodes {
        let mut idom = usize::MIN;
        let mut changed = false;
        let node_id = node.borrow().id;

        for dom_id in node.borrow().dom_set.iter() {
            if *dom_id != node_id {
                changed = true;
                idom = std::cmp::max(idom, *dom_id);
            }
        }

        // If we found an immediate dominator, set it in the node, Otherwise
        // just leave it as None.
        if changed {
            node.borrow_mut().idom = Some(idom);
        }
    }
}

/// Calculates the intersection of multiple dominance sets. This is intended
/// to be called on the predecessor list of a block. It processes the
/// preds and returns a set of block id's that are common in each
/// predecessor's dominance set.
fn dom_intersection(preds: &Vec<SifBlockRef>) -> HashSet<BlockID> {
    if preds.len() == 0 {
        return HashSet::new();
    }

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

    intersection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockID, SifBlock, SifBlockRef};
    use std::{collections::HashSet, rc::Rc};

    #[test]
    fn test_dom_calc() {
        // Based on the simple diamond shaped cfg from get_blocks(),
        // we expect the dom set to look like this:
        //
        // Node:  0    1      2      3
        // DOM:  {0} {0,1} {0, 2} {0, 3}

        let blocks = get_blocks();
        dom_calc(&blocks);

        let b0_dom_set = &blocks[0].borrow().dom_set;
        assert!(b0_dom_set.contains(&0));
        assert!(b0_dom_set.len() == 1);

        let b1_dom_set = &blocks[1].borrow().dom_set;
        assert!(b1_dom_set.contains(&0));
        assert!(b1_dom_set.contains(&1));
        assert!(b1_dom_set.len() == 2);

        let b2_dom_set = &blocks[2].borrow().dom_set;
        assert!(b2_dom_set.contains(&0));
        assert!(b2_dom_set.contains(&2));
        assert!(b2_dom_set.len() == 2);

        let b3_dom_set = &blocks[3].borrow().dom_set;
        assert!(b3_dom_set.contains(&0));
        assert!(b3_dom_set.contains(&3));
        assert!(!b3_dom_set.contains(&2));
        assert!(b3_dom_set.len() == 2);
    }

    #[test]
    fn test_idom_calc() {
        // Based on the simple diamond shaped cfg from get_blocks(),
        // we expect the dom set to look like this:
        //
        // Node:  0    1      2      3
        // DOM:  {0} {0,1} {0, 2} {0, 3}
        // IDOM:  -    0      0      0

        let blocks = get_blocks();
        dom_calc(&blocks);
        idom_calc(&blocks);

        let b0_idom = &blocks[0].borrow().idom;
        assert!(b0_idom.is_none());

        let b1_idom = &blocks[1].borrow().idom;
        assert!(b1_idom.is_some());
        assert!(b1_idom.unwrap() == 0);

        let b2_idom = &blocks[2].borrow().idom;
        assert!(b2_idom.is_some());
        assert!(b2_idom.unwrap() == 0);

        let b3_idom = &blocks[3].borrow().idom;
        assert!(b3_idom.is_some());
        assert!(b3_idom.unwrap() == 0);
    }

    /// Build a simple 4 node cfg that looks like this:
    ///
    ///    0
    ///   / \
    ///  1   2
    ///   \ /
    ///    3
    fn get_blocks() -> Vec<SifBlockRef> {
        let b0 = SifBlock::new("b0", 0);
        let b1 = SifBlock::new("b1", 1);
        let b2 = SifBlock::new("b2", 2);
        let b3 = SifBlock::new("b3", 3);

        b0.borrow_mut().edges.push(Rc::clone(&b1));
        b0.borrow_mut().edges.push(Rc::clone(&b2));
        b1.borrow_mut().edges.push(Rc::clone(&b3));
        b2.borrow_mut().edges.push(Rc::clone(&b3));

        b1.borrow_mut().preds.push(Rc::clone(&b0));
        b2.borrow_mut().preds.push(Rc::clone(&b0));
        b3.borrow_mut().preds.push(Rc::clone(&b1));
        b3.borrow_mut().preds.push(Rc::clone(&b2));
        b3.borrow_mut().preds.push(Rc::clone(&b0));

        vec![b0, b1, b2, b3]
    }
}
