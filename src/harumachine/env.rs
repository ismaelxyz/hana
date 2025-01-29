//! Provides the stack frame for the virtual machine

use super::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[repr(C)]
#[derive(Clone, Default)]
/// Stack frame for the virtual machine
pub struct Env {
    /// Cached number of args the function was called with
    pub nargs: u16,

    /// Instruction pointer to return to on Ret
    pub retip: u32,

    /// Local variable storage
    ///
    /// Slot indexes access SHOULD be bounded
    /// whenever the script is compiled to bytecode
    pub slots: HashMap<u16, Value>,

    /// Lexical parent of the current environment
    ///
    /// This is used for getting values on the previous stack frame.
    pub lexical_parent: Rc<RefCell<Option<Env>>>,
}

impl Env {
    pub fn new(retip: u32, lexical_parent: Rc<RefCell<Option<Env>>>, nargs: u16) -> Env {
        Env {
            slots: HashMap::new(),
            nargs,
            lexical_parent,
            retip,
        }
    }

    pub fn copy(other: Rc<RefCell<Option<Env>>>) -> Env {
        let reference = Rc::clone(&other);

        let mut slots = HashMap::new();
        let mut lexical_parent = Rc::new(RefCell::new(None));

        if let Some(base) = reference.borrow().as_ref() {
            slots = base.slots.clone();
            lexical_parent = base.lexical_parent.clone();
        }

        Env {
            slots,
            nargs: 0,
            lexical_parent,
            retip: u32::MAX,
        }
    }

    pub fn get(&self, idx: u16) -> Option<Value> {
        self.slots.get(&idx).cloned()
    }

    pub fn get_up(&self, up: u16, idx: u16) -> Option<Value> {
        if let Some(lexical_parent) = self.lexical_parent.borrow().as_ref() {
            if up == 1 {
                lexical_parent.get(idx)
            } else {
                lexical_parent.get_up(up - 1, idx)
            }
        } else {
            None
        }
    }

    pub fn set(&mut self, idx: u16, val: Value) {
        self.slots.insert(idx, val);
    }
}
