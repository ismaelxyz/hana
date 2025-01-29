//! Provides Env record for getting and setting environment variables
use crate::harumachine::{vm::Vm, record::Record, value::Value};
use std::{borrow::Borrow, env};

#[hana_function]
fn get(key: Value::Str) -> Value {
    match env::var(key.as_ref().borrow() as &String) {
        Ok(value) => Value::Str((*vm).borrow().malloc(value.into())),
        Err(_) => Value::Nil,
    }
}

#[hana_function]
fn set(key: Value::Str, val: Value::Str) -> Value {
    env::set_var(
        key.as_ref().borrow() as &String,
        val.as_ref().borrow() as &String,
    );

    Value::Nil
}

#[hana_function]
fn vars() -> Value {
    let mut record = (*vm).borrow().malloc(Record::new());
    for (key, value) in env::vars() {
        record
            .inner_mut_ptr()
            .insert(key, Value::Str((*vm).borrow().malloc(value.into())));
    }

    Value::Record(record)
}
