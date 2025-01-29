//! Provides Int record for handling integers
use std::borrow::Borrow;
use std::path::PathBuf;

use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;

#[hana_function()]
fn constructor(path: Value::Str) -> Value {
    let mut rec = (*vm).borrow().malloc(Record::new());
    let path = path.as_ref().borrow() as &String;
    rec.inner_mut_ptr().native_field = Some(Box::new(PathBuf::from(path)));
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record((*vm).borrow().stdlib.as_ref().unwrap().dir_rec.clone()),
    );
    Value::Record(rec)
}

#[hana_function()]
fn ls(dir: Value::Record) -> Value {
    let field = dir.as_ref().native_field.as_ref().unwrap();
    let dir = field.downcast_ref::<PathBuf>().unwrap();
    let mut entries = (*vm).borrow().malloc(Vec::new());
    let read_dir = if let Ok(read_dir) = std::fs::read_dir(dir) {
        read_dir
    } else {
        return Value::Array(entries);
    };
    for entry in read_dir.flatten() {
        if let Some(path) = entry.path().to_str() {
            entries
                .inner_mut_ptr()
                .push(Value::Str((*vm).borrow().malloc(path.to_string().into())));
        }
    }
    Value::Array(entries)
}
