//! Provides File record for handling files
use std::borrow::Borrow;
use std::boxed::Box;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::harumachine::{record::Record, value::Value, vm::Vm, vmerror::VmError};

#[hana_function]
fn constructor(path: Value::Str, mode: Value::Str) -> Value {
    // options
    let mut options = OpenOptions::new();
    for ch in mode.as_ref().chars() {
        match ch {
            'r' => options.read(true),
            'w' => options.write(true),
            'c' => options.create(true),
            'n' => options.create_new(true),
            'a' => options.append(true),
            't' => options.truncate(true),
            _ => {
                panic!("expected options");
            }
        };
    }

    // file object
    let mut rec = (*vm).borrow().malloc(Record::new());
    // store native file
    match options.open(path.as_ref().borrow() as &String) {
        Ok(file) => {
            rec.inner_mut_ptr().native_field = Some(Box::new(file));
        }
        Err(err) => {
            rec.inner_mut_ptr().insert(
                "prototype",
                Value::Record((*vm).borrow().stdlib.as_ref().unwrap().io_error.clone()),
            );
            rec.inner_mut_ptr().insert(
                "why",
                Value::Str((*vm).borrow().malloc(format!("{:?}", err).into())),
            );
            rec.inner_mut_ptr().insert("where", Value::Str(path));
            hana_raise!(vm, Value::Record(rec));
        }
    }
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record((*vm).borrow().stdlib.as_ref().unwrap().file_rec.clone()),
    );
    rec.inner_mut_ptr().insert("path", Value::Str(path));
    rec.inner_mut_ptr().insert("mode", Value::Str(mode));
    Value::Record(rec)
}

// reopen
#[hana_function]
fn close(mut file: Value::Record) -> Value {
    file.inner_mut_ptr().native_field = None;
    Value::Nil
}

// read
#[hana_function]
fn read(mut file: Value::Record) -> Value {
    let field = file.inner_mut_ptr().native_field.as_mut().unwrap();
    let file = field.downcast_mut::<File>().unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    Value::Str((*vm).borrow().malloc(data.into()))
}

#[hana_function]
fn read_up_to(mut file: Value::Record, n: Value::Int) -> Value {
    let field = file.inner_mut_ptr().native_field.as_mut().unwrap();
    let file = field.downcast_mut::<File>().unwrap();
    let mut bytes: Vec<u8> = vec![0; n as usize];
    file.read_exact(&mut bytes).unwrap();
    Value::Str(
        (*vm).borrow().malloc(
            String::from_utf8(bytes)
                .unwrap_or_else(|e| panic!("error decoding file: {:?}", e))
                .into(),
        ),
    )
}

// write
#[hana_function]
fn write(mut file: Value::Record, buf: Value::Str) -> Value {
    let file = file.inner_mut_ptr();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        Value::Int(file.write_all(buf.as_ref().as_bytes()).is_ok() as i64)
    } else {
        Value::Int(0)
    }
}

// positioning
#[hana_function]
fn seek(mut file: Value::Record, pos: Value::Int) -> Value {
    let file = file.inner_mut_ptr();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Ok(result) = file.seek(SeekFrom::Current(pos)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else {
        Value::Int(-1)
    }
}

#[hana_function]
fn seek_from_start(mut file: Value::Record, pos: Value::Int) -> Value {
    let file = file.inner_mut_ptr();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Ok(result) = file.seek(SeekFrom::Start(pos as u64)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else {
        Value::Int(-1)
    }
}

#[hana_function]
fn seek_from_end(mut file: Value::Record, pos: Value::Int) -> Value {
    let file = file.inner_mut_ptr();
    if let Some(field) = file.native_field.as_mut() {
        let file = field.downcast_mut::<File>().unwrap();
        if let Ok(result) = file.seek(SeekFrom::End(pos)) {
            Value::Int(result as i64)
        } else {
            Value::Int(-1)
        }
    } else {
        Value::Int(-1)
    }
}
