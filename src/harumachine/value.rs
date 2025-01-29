//! Provides an abstraction for native values

use super::function::Function;
use super::gc::{ref_dec, ref_inc, Gc};
// use super::nativeval::{NativeValue, NativeValueType};
use super::record::Record;
use super::string::HaruString;
use super::vm::Vm;
use std::borrow::Borrow;

pub type NativeFnData = fn(Rc<RefCell<Vm>>, u16);

#[derive(Clone)]
pub enum Value {
    Nil,
    True,
    False,
    // Bool(bool),
    Int(i64),
    Float(f64),
    NativeFn(NativeFnData),
    Fn(Gc<Function>),
    Str(Gc<HaruString>),
    Record(Gc<Record>),
    Array(Gc<Vec<Value>>),

    // this is temporary while I correct the errors, then I will give it a specific type.
    //RuntimeError(Gc<HaruString>),
    InterpreterError,

    PropagateError,
    Iterator,
}

impl PartialEq<Value> for Value {
    // Required method
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil)
            | (True, True)
            | (False, False)
            | (InterpreterError, InterpreterError)
            | (PropagateError, PropagateError)
            | (Iterator, Iterator) => true,

            // Bool(bool),
            (Int(left), Int(right)) => left == right,
            (Float(left), Float(right)) => left == right,
            (NativeFn(native_fnl), NativeFn(native_fnr)) => native_fnl == native_fnr,
            (Fn(gcl), Fn(gcr)) => std::ptr::eq(gcl.to_raw(), gcr.to_raw()),
            (Str(gcl), Str(gcr)) => std::ptr::eq(gcl.to_raw(), gcr.to_raw()),
            (Record(gcl), Record(gcr)) => std::ptr::eq(gcl.to_raw(), gcr.to_raw()),
            (Array(gcl), Array(gcr)) => std::ptr::eq(gcl.to_raw(), gcr.to_raw()),

            // (RuntimeError(gcl), RuntimeError(gcr)) => std::ptr::eq(gcl.to_raw(), gcr.to_raw()),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_size = std::mem::size_of_val(self);
        let other_size = std::mem::size_of_val(other);

        self_size.cmp(&other_size)
    }
}

// boolean?
#[no_mangle]
pub(super) fn value_is_true(value: Value) -> bool {
    match value {
        Value::Int(i) => i > 0,
        Value::Float(f) => f > 0.0,
        Value::Str(s) => unsafe { (*s.to_raw()).is_empty() },
        _ => false,
    }
}

impl Value {

    pub fn as_gc_pointer(&self) -> Option<*mut libc::c_void> {

        match &self {
            Value::Fn(gc) => Some(gc.to_raw() as _),
            Value::Str(gc) => Some(gc.to_raw() as _),
            Value::Record(gc) => Some(gc.to_raw() as _),
            Value::Array(gc) => Some(gc.to_raw() as _),
            //Value::RuntimeError(gc) => Some(gc.to_raw() as _),
            _ => None,
        }
    }

    pub fn ref_inc(&self) {
        if let Some(ptr) = self.as_gc_pointer() {
            unsafe {
                ref_inc(ptr);
            }
        }
    }
    pub fn ref_dec(&self) {
        if let Some(ptr) = self.as_gc_pointer() {
            unsafe {
                ref_dec(ptr);
            }
        }
    }

    pub fn get_prototype(&self, vm: Rc<RefCell<Vm>>) -> Option<Gc<Record>> {
        crate::harumachine::inside::get_prototype(vm, self.clone())
    }

    pub fn is_true(&self) -> bool {
        value_is_true(self.clone())
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Nil => "nil",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::NativeFn(_) | Value::Fn(_) => "Function",
            Value::Str(_) => "String",
            Value::Record(_) => "Record",
            Value::Array(_) => "Array",
            _ => "unk",
        }
    }
}

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::True => write!(f, "1"),
            Value::False => write!(f, "0"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(_) => write!(f, "[native fn]"),
            Value::Fn(_) => write!(f, "[fn]"),
            Value::Str(p) => write!(f, "{}", p.as_ref().borrow() as &String),
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(a) => unsafe {
                let a = &*a.to_raw();
                write!(f, "[")?;
                if !a.is_empty() {
                    write!(f, "{}", a[0])?;
                }

                for item in a.iter().skip(1) {
                    write!(f, ", {}", item)?;
                }
                write!(f, "]")
            },
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::NativeFn(nf) => write!(f, "[native function {:p}]", nf),
            Value::Fn(xf) => write!(f, "[function {:p}]", xf.to_raw()),
            Value::Str(p) => {
                let mut s = String::new();
                let p = p.as_ref().borrow();
                for ch in (p as &String).chars() {
                    match ch {
                        '\n' => s.push_str("\\n"),
                        '"' => s.push('"'),
                        _ => s.push(ch),
                    }
                }
                write!(f, "\"{}\"", s)
            }
            Value::Record(p) => write!(f, "[record {:p}]", p.to_raw()),
            Value::Array(p) => write!(f, "[array {:p}]", p.to_raw()),
            _ => write!(f, "[unk]"),
        }
    }
}
