//! Provides an abstraction for native values

use super::function::Function;
use super::gc::Gc;
use super::nativeval::{NativeValue, NativeValueType};
use super::record::Record;
use super::string::HaruString;
use super::vm::Vm;
use std::borrow::Borrow;

pub type NativeFnData = unsafe extern "C" fn(*mut Vm, u16);

#[derive(Clone, PartialEq)]
pub enum Value {
    // we don't have control over how rust manages its variant
    // types, so this is a convenient wrapper for (de)serialising
    // hana's values
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
    Array(Gc<Vec<NativeValue>>),

    PropagateError,
    Iterator,
}

// boolean?
#[no_mangle]
pub(super) extern "C" fn value_is_true(left: NativeValue) -> bool {
    match unsafe { left.unwrap() } {
        Value::Int(i) => i > 0,
        Value::Float(f) => f > 0.0,
        Value::Str(s) => unsafe { (*s.to_raw()).is_empty() },
        _ => false,
    }
}

impl Value {
    // wrapper for native
    pub fn wrap(&self) -> NativeValue {
        match &self {
            Value::Nil => NativeValue {
                r#type: NativeValueType::TYPE_NIL,
                data: 0,
            },
            Value::True => NativeValue {
                r#type: NativeValueType::TYPE_INT,
                data: 1,
            },
            Value::False => NativeValue {
                r#type: NativeValueType::TYPE_INT,
                data: 0,
            },
            Value::Int(n) => NativeValue {
                r#type: NativeValueType::TYPE_INT,
                data: *n as u64,
            },
            Value::Float(n) => NativeValue {
                r#type: NativeValueType::TYPE_FLOAT,
                data: n.to_bits(),
            },
            Value::NativeFn(f) => NativeValue {
                r#type: NativeValueType::TYPE_NATIVE_FN,
                data: *f as u64,
            },
            Value::Fn(p) => NativeValue {
                r#type: NativeValueType::TYPE_FN,
                data: p.to_raw() as u64,
            },
            Value::Str(p) => NativeValue {
                r#type: NativeValueType::TYPE_STR,
                data: p.to_raw() as u64,
            },
            Value::Record(p) => NativeValue {
                r#type: NativeValueType::TYPE_DICT,
                data: p.to_raw() as u64,
            },
            Value::Array(p) => NativeValue {
                r#type: NativeValueType::TYPE_ARRAY,
                data: p.to_raw() as u64,
            },
            Value::Iterator => NativeValue {
                r#type: NativeValueType::TYPE_INTERPRETER_ITERATOR,
                data: 0,
            },
            _ => unimplemented!(),
        }
    }

    pub unsafe fn get_prototype(&self, vm: *const Vm) -> *const Record {
        crate::harumachine::inside::get_prototype(&*vm, self.wrap())
    }

    pub fn is_true(&self) -> bool {
        value_is_true(self.wrap())
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

use std::fmt;

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
                    write!(f, "{}", a[0].unwrap())?;
                }

                for item in a.iter().skip(1) {
                    write!(f, ", {}", item.unwrap())?;
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
