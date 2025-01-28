//! Provides a function value in Hana

use super::env::Env;
use crate::vmbindings::gc::{push_gray_body, GcNode, GcTraceable};
use std::{cell::RefCell, rc::Rc};

// TODO: add a name type attribute to know the name of the function
#[repr(C)]
#[derive(Clone)]
pub struct Function {
    /// Starting instruction pointer of the function
    pub ip: u32,
    /// Number of args the function takes in
    pub nargs: u16,

    // internal rust properties:
    /// Represents the current local environment
    /// at the time the function is declared.
    ///
    /// This will be COPIED into another struct env whenever OP_CALL is issued.
    ///
    /// Wwe use this to implement closures.
    pub bound: Rc<RefCell<Option<Env>>>,
}

impl Function {
    pub unsafe fn new(ip: u32, nargs: u16, env: Rc<RefCell<Option<Env>>>) -> Function {
        let bound = match env.borrow().is_some() {
            true => Env::copy(Rc::clone(&env)),
            false => Env::new(0, Rc::default(), nargs),
        };

        Function {
            ip,
            nargs,
            bound: Rc::new(RefCell::new(Some(bound))),
        }
    }

    pub fn get_bound(&self) -> Rc<RefCell<Option<Env>>> {
        Rc::clone(&self.bound)
    }
}

// gc traceable
impl GcTraceable for Function {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        let Some(bound) = self.bound.borrow_mut().take() else {
            return;
        };

        for val in bound.slots.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }

        *self.bound.borrow_mut() = Some(bound);
    }
}
