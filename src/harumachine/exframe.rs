//! Provides an exception frame interface for storing try..case data
use std::cell::RefCell;
use std::collections::BTreeMap;
//use std::ptr::NonNull;
use std::rc::Rc;

use super::env::Env;
use super::function::Function;
use super::record::Record;
use super::value::Value;
use super::vm::Vm;

/// Exception frame
#[derive(Clone)]
pub struct ExFrame {
    /// Exception frame handlers
    handlers: BTreeMap<*const Record, Function>,
    /// The target call stack frame to rewind to
    #[allow(dead_code)]
    pub unwind_env: Option<Rc<RefCell<Option<Env>>>>,
    /// The target virtual machine stack index to rewind to
    #[allow(dead_code)]
    pub unwind_stack: usize,
    /// How many native functions to return until we can call this?
    pub unwind_native_call_depth: usize,
}

impl ExFrame {
    pub fn new(
        unwind_env: Option<Rc<RefCell<Option<Env>>>>,
        unwind_stack: usize,
        unwind_native_call_depth: usize,
    ) -> ExFrame {
        ExFrame {
            handlers: BTreeMap::new(),
            unwind_env,
            unwind_stack,
            unwind_native_call_depth,
        }
    }

    pub fn set_handler(&mut self, rec: &Record, fun: Function) {
        self.handlers.insert(rec, fun);
    }

    pub fn get_handler(&self, vm: &Vm, val: &Value) -> Option<&Function> {
        let rec = unsafe { val.get_prototype(vm) };
        self.handlers.get(&rec)
    }
}
