//! Provides Record record for handling records
use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;

#[hana_function()]
fn constructor() -> Value {
    Value::Record((*vm).borrow().malloc(Record::new()))
}

#[hana_function()]
fn keys(rec: Value::Record) -> Value {
    let mut array = (*vm).borrow().malloc(Vec::new());
    for (key, _) in rec.as_ref().iter() {
        array
            .inner_mut_ptr()
            .push(Value::Str((*vm).borrow().malloc(key.clone())));
    }

    Value::Array(array)
}

#[hana_function()]
fn has_key(rec: Value::Record, needle: Value::Str) -> Value {
    for (key, _) in rec.as_ref().iter() {
        if key == needle.as_ref() {
            return Value::True;
        }
    }

    Value::False
}
