use sifc_compiler::sifv::SifVal;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Heap {
    initial_size: usize,
    alloc_count: usize,
    heap: HashMap<String, SifVal>,
}

impl Heap {
    pub fn new(initial_size: usize) -> Heap {
        Heap {
            initial_size: initial_size,
            alloc_count: 1,
            heap: HashMap::with_capacity(initial_size),
        }
    }

    pub fn get(&mut self, key: &String) -> Option<&SifVal> {
        self.heap.get(key)
    }

    pub fn set(&mut self, key: String, val: SifVal) {
        self.heap.insert(key, val);
    }
}
