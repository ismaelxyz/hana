//! Provides Int record for handling integers
use std::borrow::Borrow;
use std::path::PathBuf;

use crate::vmbindings::record::Record;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;

#[hana_function()]
fn constructor(path: Value::Str) -> Value {
    let mut rec = vm.malloc(Record::new());
    let path = path.as_ref().borrow() as &String;
    rec.inner_mut_ptr().native_field = Some(Box::new(PathBuf::from(path)));
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record(vm.stdlib.as_ref().unwrap().dir_rec.clone()).wrap(),
    );
    Value::Record(rec)
}

#[hana_function()]
fn ls(dir: Value::Record) -> Value {
    let field = dir.as_ref().native_field.as_ref().unwrap();
    let dir = field.downcast_ref::<PathBuf>().unwrap();
    let mut entries = vm.malloc(Vec::new());
    let read_dir = if let Ok(read_dir) = std::fs::read_dir(dir) {
        read_dir
    } else {
        return Value::Array(entries);
    };
    for entry in read_dir.flatten() {
        if let Some(path) = entry.path().to_str() {
            entries
                .inner_mut_ptr()
                .push(Value::Str(vm.malloc(path.to_string().into())).wrap());
        }
    }
    Value::Array(entries)
}
