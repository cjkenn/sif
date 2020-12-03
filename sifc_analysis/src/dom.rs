use crate::block::{BlockID, SifBlockRef};
use std::{collections::HashSet, rc::Rc};

/// Calculate dominance lists (and dominance sets) for each node in nodes.
/// This function will update each node in place with the correct dominance information.
pub(crate) fn dom_calc(nodes: &Vec<SifBlockRef>) {
    // Initial conditions:
    // 1. The entry node's dominator set includes only itself.
    // 2. For all other nodes (ie. not the entry), the dominator set
    //    includes all nodes.

    // Set the dominators for the entry node, subject to initial conditions.
    nodes[0].borrow_mut().dom = vec![Rc::clone(&nodes[0])];
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

            // The temp dom set is the current node unioned with the intersection
            // of dominators in this node's predecessors.
            let mut pred_dom_intersection = dom_intersection(&node.borrow().preds);
            pred_dom_intersection.insert(0, Rc::clone(&node));

            if dom_equal(&pred_dom_intersection, &node.borrow().dom) {
                node.borrow_mut().dom = pred_dom_intersection.clone();
                let new_dom_set: HashSet<BlockID> = pred_dom_intersection
                    .iter()
                    .map(|k| k.borrow().id)
                    .collect();
                node.borrow_mut().dom_set = new_dom_set;
                changed = true;
            }

            i += 1;
        }
    }
}

/// Calculates the intersection of multiple dominance sets. This is intended
/// to be called on the predecessor list of a block. It processes the
/// preds and returns a vector of block refs that are common in each
/// predecessor's dominance set.
fn dom_intersection(preds: &Vec<SifBlockRef>) -> Vec<SifBlockRef> {
    if preds.len() == 0 {
        return Vec::new();
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

    let mut result = Vec::new();
    for pred in preds {
        let id = pred.borrow().id;
        if intersection.contains(&id) {
            result.push(Rc::clone(&pred));
        }
    }

    result
}

/// Check if two dominator lists are equal by collecting the lists into sets
/// of block id and checking set equality. This needs to be done
/// because vector equality accounts for order, and the sets are considered the same
/// even if the order is different.
/// O(n) time and O(n) space.
fn dom_equal(dom1: &Vec<SifBlockRef>, dom2: &Vec<SifBlockRef>) -> bool {
    let dom_set1: HashSet<BlockID> = dom1.iter().map(|k| k.borrow().id).collect();
    let dom_set2: HashSet<BlockID> = dom2.iter().map(|k| k.borrow().id).collect();

    dom_set1 == dom_set2
}
