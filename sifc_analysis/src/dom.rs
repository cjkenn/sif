use crate::block::{BlockID, SifBlockRef};
use std::{collections::HashSet, rc::Rc};

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
