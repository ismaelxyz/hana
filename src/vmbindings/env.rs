//! Provides the stack frame for the virtual machine

use std::cell::RefCell;
use std::rc::Rc;

use super::nativeval::NativeValue;
use super::value::Value;

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
    pub slots: Vec<NativeValue>,

    /// Lexical parent of the current environment
    ///
    /// This is used for getting values on the previous stack frame.
    pub lexical_parent: Rc<RefCell<Option<Env>>>,
}

impl Env {
    pub fn new(retip: u32, lexical_parent: Rc<RefCell<Option<Env>>>, nargs: u16) -> Env {
        Env {
            slots: Vec::new(),
            nargs,
            lexical_parent,
            retip,
        }
    }

    pub fn copy(other: Rc<RefCell<Option<Env>>>) -> Env {
        let reference = Rc::clone(&other);

        let mut slots = Vec::new();
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
 /// # Safety
 /// 
 /// This should be fixed to return a runtime error in the interpreter.
    pub unsafe fn get(&self, idx: u16) -> NativeValue {
        *self.slots.get_unchecked(idx as usize)
    }

    pub unsafe fn get_up(&self, up: u16, idx: u16) -> NativeValue {
        if let Some(lexical_parent) = self.lexical_parent.borrow().as_ref() {
            if up == 1 {
                lexical_parent.get(idx)
            } else {
                lexical_parent.get_up(up - 1, idx)
            }
        } else {
            // TODO: This can be replaced by an error from the interpreter
            panic!("this should never happen")
        }
    }

    pub unsafe fn set(&mut self, idx: u16, val: NativeValue) {
        let elem = self.slots.get_unchecked_mut(idx as usize);
        *elem = val;
    }

    pub fn reserve(&mut self, nslots: u16) {
        self.slots.resize(nslots as usize, Value::Nil.wrap());
    }
}
