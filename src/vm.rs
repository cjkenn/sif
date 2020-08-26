use crate::error::VMErr;
use crate::block::SifBlock;
use crate::op::OpCode;
use crate::val::SifVal;

const STACK_MAX: usize = 256;

pub struct VM<'v> {
    blocks: Vec<SifBlock>,
    stack: Vec<&'v dyn SifVal>,
    top: usize,
    ip: usize,
}

impl<'v> VM<'v> {
    pub fn new() -> VM<'v> {
        VM {
            blocks: Vec::new(),
            stack: Vec::with_capacity(STACK_MAX),
            top: 0,
            ip: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), VMErr> {
        loop {
            let curr_block = &self.blocks[self.ip];

            for instr in &curr_block.instrs {
                match instr {
                    OpCode::Ret{..} => {
                        return Ok(());
                    },
                    OpCode::Const { val } => {
                        self.push_val(val);
                        return Ok(());
                    },
                    _ => { return Ok(()); }
                }
            }

            self.ip = self.ip + 1;
        }
    }

    fn reset_stack(&mut self) {
        self.top = 0;
        self.stack.clear();
    }

    fn push_val(&mut self, vl: &'v dyn SifVal) {
        self.stack.push(vl);
        self.top = self.top + 1;
    }

    fn pop_val(&mut self) -> &'v dyn SifVal {
        // TODO: deal with empty stack
        self.top = self.top - 1;
        self.stack.pop().unwrap()
    }
}
