use crate::ast::AstNode;
use std::collections::HashMap;

pub enum SymTy {
    Fn,
    Var,
}

/// Scope is a map of ident names to their declaration AST.
type Scope = HashMap<String, AstNode>;

#[derive(Debug)]
pub struct SymTab {
    /// Current scope level in the symbol table. 0 is the global scope,
    /// and when the table is created we allocate a new hashmap to hold that scope.
    /// (ie. manually creating the global scope after creating this struct is NOT required).
    curr_lvl: usize,

    /// The actual symbol table, as a stack of maps. Each new block scope is pushed onto
    /// this stack, and popped off/finalized when we exit the block.
    tab: Vec<Scope>,
}

impl SymTab {
    pub fn new() -> SymTab {
        SymTab {
            curr_lvl: 0,
            tab: vec![HashMap::new()],
        }
    }

    /// Creates a new scope and pushes it onto the scope stack.
    /// This should be called at the entry of each block in order to properly
    /// block scope statements.
    pub fn init_scope(&mut self) {
        self.curr_lvl = self.curr_lvl + 1;
        self.tab.push(HashMap::new());
    }

    /// Close the current scope block, moving back up into a higher
    /// (previous) scope.
    pub fn close_scope(&mut self) {
        self.curr_lvl = self.curr_lvl - 1;
    }

    /// True if the current scope is the global scope, false otherwise.
    pub fn is_global(&self) -> bool {
        self.curr_lvl == 0
    }

    /// Return the current scope level in the non-finalized table.
    pub fn level(&self) -> usize {
        self.curr_lvl
    }

    /// Store a symbol in the table at the current level.
    pub fn store(&mut self, key: &str, ast: AstNode) {
        self.tab[self.curr_lvl].insert(String::from(key), ast);
    }

    /// Get a symbol from the table. We check the current scope and
    /// all parent scopes for the symbol. Returns None if the symbol
    /// doesn't exist.
    pub fn retrieve(&self, key: &str) -> Option<AstNode> {
        let mut curr = self.curr_lvl;

        // This loop always breaks: if we find the symbol we immediately
        // return out of the loop. If we don't find it, we decrement the scope
        // and look at a higher level. If we find nothing there, we continue
        // UNLESS the current scope level is 0 (the top level scope). In that
        // case, we break and return None.
        loop {
            match self.tab[curr].get(key) {
                Some(val) => {
                    return Some(val.clone());
                }
                None if curr == 0 => {
                    break;
                }
                None => (),
            };
            curr = curr - 1;
        }

        None
    }

    pub fn contains(&self, key: &str) -> bool {
        self.retrieve(key).is_some()
    }
}
