use sifc_bytecode::sifv::SifVal;
use std::{cell::RefCell, rc::Rc};

type DRegVec = Vec<Rc<RefCell<DReg>>>;

/// DReg represents a data register.
#[derive(Clone, Debug)]
pub struct DReg {
    pub name: String,
    pub cont: Option<SifVal>,
}

impl DReg {
    pub fn new(n: String) -> DReg {
        DReg {
            name: n,
            cont: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataRegisterList {
    initial_size: usize,
    alloc_count: usize,
    dregs: DRegVec,
}

impl DataRegisterList {
    pub fn init(initial_size: usize) -> DataRegisterList {
        let mut regs = Vec::with_capacity(initial_size);

        for i in 0..initial_size {
            let reg = DReg::new(format!("r{}", i));
            regs.push(Rc::new(RefCell::new(reg)));
        }

        DataRegisterList {
            initial_size: initial_size,
            alloc_count: 0,
            dregs: regs,
        }
    }

    pub fn get(&mut self, index: usize) -> Rc<RefCell<DReg>> {
        if index >= self.dregs.len() {
            self.expand(index);
        }
        Rc::clone(&self.dregs[index])
    }

    pub fn set_contents(&mut self, index: usize, val: Option<SifVal>) {
        if index >= self.dregs.len() {
            self.expand(index);
        }
        self.dregs[index].borrow_mut().cont = val;
    }

    pub fn register_count(&self) -> usize {
        self.dregs.len()
    }

    fn expand(&mut self, upto: usize) {
        self.alloc_count += 1;

        let max_size = self.register_count() * self.alloc_count;
        let grow_size = if max_size < upto { upto } else { max_size };
        self.dregs.reserve(grow_size);

        let mut start_idx = self.dregs.len() - 1;
        for _i in 0..grow_size {
            let reg = DReg::new(format!("r{}", start_idx));
            self.dregs.push(Rc::new(RefCell::new(reg)));
            start_idx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand() {
        // Initialize with 1 register
        let mut initial = DataRegisterList::init(1);
        assert!(initial.register_count() == 1);

        // When we get register at index 1, we must expand.
        let _i = initial.get(1);
        assert!(initial.register_count() == 2);

        // In this case, we expand to reg_count() * alloc_count (2 * 3)
        let _x = initial.get(2);
        assert!(initial.register_count() == 6);

        // No expansion here, becauase register at index 3 is within bounds.
        let _x = initial.get(3);
        assert!(initial.register_count() == 6);

        // Here we expand upto the desired register + previous memory.
        let _i = initial.get(100);
        assert!(initial.register_count() == 106);
    }
}
