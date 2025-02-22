//! Provides a record value in Hana

use super::gc::{push_gray_body, GcNode, GcTraceable};
use super::hmap::HaruHashMap;
use super::string::HaruString;
use super::value::Value;
use std::any::Any;
use std::borrow::Borrow;
use std::boxed::Box;
use std::hash::Hash;

/// A record value in Hana
#[derive(Default)]
pub struct Record {
    data: HaruHashMap,
    prototype: Option<&'static Record>,
    // it says static but it lasts as long as Record, see below!
    /// Dynamic field for use in native functions
    pub native_field: Option<Box<dyn Any>>,
}

impl Record {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(n: usize) -> Record {
        Record {
            data: HaruHashMap::with_capacity(n),
            prototype: None,
            native_field: None,
        }
    }

    pub fn get<T>(&self, k: &T) -> Option<&Value>
    where
        HaruString: Borrow<T>,
        T: Hash + Eq + ?Sized,
    {
        if let Some(v) = self.data.get(k) {
            return Some(v);
        } else if let Some(prototype) = self.prototype {
            return prototype.get(k);
        }
        None
    }

    pub fn insert<K>(&mut self, k: K, v: Value)
    where
        K: Into<HaruString> + Hash + Eq,
    {
        let k = k.into();
        if (k.borrow() as &String) == "prototype" {
            self.prototype = unsafe {
                match &v {
                    // since the borrow checker doesn't know that self.prototype
                    // can last as long as self, we'll have to use unsafe
                    Value::Record(x) => Some(&*x.to_raw()),
                    _ => None,
                }
            };
        }
        self.data.insert(k, v);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<HaruString, Value> {
        self.data.iter()
    }

    // But why the prototype of the prototype of the prototype of the...
    // Should I remove the while?
    pub fn is_prototype_of(&self, other: &Record) -> bool {
        let mut prototype = self.prototype;
        while prototype.is_some() {
            let proto = prototype.unwrap();
            if std::ptr::eq(proto, other) {
                return true;
            }
            // Father of the Father...
            prototype = proto.prototype;
        }
        false
    }
}

impl GcTraceable for Record {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        for (_, val) in self.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }
    }
}
