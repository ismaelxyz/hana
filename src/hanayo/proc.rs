//! Provides Proc record for handling child process spawned by Cmd
use std::cell::RefCell;
use std::io::Write;
use std::process::Child;
use std::rc::Rc;

use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;
use crate::harumachine::vmerror::VmError;

// inputs
#[hana_function()]
fn in_(mut process: Value::Record, input: Value::Str) -> Value {
    let field = process.inner_mut_ptr().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    p.stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_ref().as_bytes())
        .unwrap();

    Value::Record(process)
}

fn utf8_decoding_error(err: std::string::FromUtf8Error, vm: Rc<RefCell<Vm>>) -> Value {
    let mut rec = (*vm).borrow().malloc(Record::new());
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record(
            (*vm)
                .borrow()
                .stdlib
                .as_ref()
                .unwrap()
                .utf8_decoding_error
                .clone(),
        ),
    );
    rec.inner_mut_ptr().insert(
        "why",
        Value::Str((*vm).borrow().malloc(format!("{:?}", err).into())),
    );
    rec.inner_mut_ptr().insert("where", Value::Int(0));
    Value::Record(rec)
}

// outs
#[hana_function()]
fn out(mut process: Value::Record) -> Value {
    // stdout as string
    let p = *process
        .inner_mut_ptr()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    match String::from_utf8(out.stdout) {
        Ok(s) => Value::Str((*vm).borrow().malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
}

#[hana_function()]
fn err(mut process: Value::Record) -> Value {
    // stderr as string
    let p = *process
        .inner_mut_ptr()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    match String::from_utf8(out.stderr) {
        Ok(s) => Value::Str((*vm).borrow().malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
}

#[hana_function()]
fn outputs(mut process: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let p = *process
        .inner_mut_ptr()
        .native_field
        .take()
        .unwrap()
        .downcast::<Child>()
        .unwrap();
    let out = p.wait_with_output().unwrap();
    let mut arr = (*vm).borrow().malloc(Vec::new());
    match String::from_utf8(out.stdout) {
        Ok(s) => arr
            .inner_mut_ptr()
            .push(Value::Str((*vm).borrow().malloc(s.into()))),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
    match String::from_utf8(out.stderr) {
        Ok(s) => arr
            .inner_mut_ptr()
            .push(Value::Str((*vm).borrow().malloc(s.into()))),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
    Value::Array(arr)
}

// other
#[hana_function()]
fn wait(mut process: Value::Record) -> Value {
    let field = process.inner_mut_ptr().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.wait() {
        Ok(e) => {
            if let Some(code) = e.code() {
                Value::Int(code as i64)
            } else {
                Value::Int(0)
            }
        }
        Err(_) => Value::Nil,
    }
}

#[hana_function()]
fn kill(mut process: Value::Record) -> Value {
    let field = process.inner_mut_ptr().native_field.as_mut().unwrap();
    let p = field.downcast_mut::<Child>().unwrap();
    match p.kill() {
        Ok(()) => Value::Int(1),
        Err(_) => Value::Int(0),
    }
}
