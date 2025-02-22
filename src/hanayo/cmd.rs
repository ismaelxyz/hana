//! Provides Cmd record for executing and handling commands
use crate::harumachine::{record::Record, value::Value, vm::Vm, vmerror::VmError};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};
use std::rc::Rc;

#[hana_function]
fn constructor(val: Value::Any) -> Value {
    let cmd: Command = match val {
        Value::Array(arr) => {
            let arr = arr.as_ref();
            if arr.is_empty() {
                let mut rec = (*vm).borrow().malloc(Record::new());
                rec.inner_mut_ptr().insert(
                    "prototype",
                    Value::Record(
                        (*vm)
                            .borrow()
                            .stdlib
                            .as_ref()
                            .unwrap()
                            .invalid_argument_error
                            .clone(),
                    ),
                );
                rec.inner_mut_ptr().insert(
                    "why",
                    Value::Str(
                        (*vm).borrow().malloc(
                            "Expected argument array to have at least 1 member"
                                .to_string()
                                .into(),
                        ),
                    ),
                );
                rec.inner_mut_ptr().insert("where", Value::Int(0));
                hana_raise!(vm, Value::Record(rec));
            }
            let mut cmd =
                Command::new(match arr[0].clone() {
                    Value::Str(s) => (s.as_ref().borrow() as &String).clone(),
                    _ => {
                        let mut rec = (*vm).borrow().malloc(Record::new());
                        rec.inner_mut_ptr().insert(
                            "prototype",
                            Value::Record(
                                (*vm)
                                    .borrow()
                                    .stdlib
                                    .as_ref()
                                    .unwrap()
                                    .invalid_argument_error
                                    .clone(),
                            ),
                        );
                        rec.inner_mut_ptr().insert(
                            "why",
                            Value::Str((*vm).borrow().malloc(
                                "Expected command to be of string type".to_string().into(),
                            )),
                        );
                        rec.inner_mut_ptr().insert("where", Value::Int(0));
                        hana_raise!(vm, Value::Record(rec));
                    }
                });
            if arr.len() > 1 {
                let slice = &arr.as_slice()[1..];
                for val in slice {
                    match val.clone() {
                        Value::Str(s) => cmd.arg((s.as_ref().borrow() as &String).clone()),
                        _ => {
                            let mut rec = (*vm).borrow().malloc(Record::new());
                            rec.inner_mut_ptr().insert(
                                "prototype",
                                Value::Record(
                                    (*vm)
                                        .borrow()
                                        .stdlib
                                        .as_ref()
                                        .unwrap()
                                        .invalid_argument_error
                                        .clone(),
                                ),
                            );
                            rec.inner_mut_ptr().insert(
                                "why",
                                Value::Str((*vm).borrow().malloc(
                                    "Expected argument to be of string type".to_string().into(),
                                )),
                            );
                            rec.inner_mut_ptr().insert("where", Value::Int(0));
                            hana_raise!(vm, Value::Record(rec));
                        }
                    };
                }
            }
            cmd
        }
        Value::Str(scmd) => {
            let mut cmd = Command::new("sh");
            cmd.arg("-c")
                .arg((scmd.as_ref().borrow() as &String).clone());
            cmd
        }
        _ => {
            let mut rec = (*vm).borrow().malloc(Record::new());
            rec.inner_mut_ptr().insert(
                "prototype",
                Value::Record(
                    (*vm)
                        .borrow()
                        .stdlib
                        .as_ref()
                        .unwrap()
                        .invalid_argument_error
                        .clone(),
                ),
            );
            rec.inner_mut_ptr().insert(
                "why",
                Value::Str(
                    (*vm).borrow().malloc(
                        "Expected argument to be of string or array type"
                            .to_string()
                            .into(),
                    ),
                ),
            );
            rec.inner_mut_ptr().insert("where", Value::Int(0));
            hana_raise!(vm, Value::Record(rec));
        }
    };
    // cmd object
    let mut rec = (*vm).borrow().malloc(Record::new());
    // store native cmd
    rec.inner_mut_ptr().native_field = Some(Box::new(cmd));
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record((*vm).borrow().stdlib.as_ref().unwrap().cmd_rec.clone()),
    );
    Value::Record(rec)
}

// inputs
#[hana_function]
fn in_(mut cmd: Value::Record, input: Value::Str) -> Value {
    cmd.inner_mut_ptr()
        .insert("input_buffer", Value::Str(input));
    Value::Record(cmd)
}

// outputs
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

// helper class
enum OutputResult {
    Process(Child),
    Output(Result<Output, std::io::Error>),
}

impl OutputResult {
    fn get_process(self) -> Child {
        match self {
            OutputResult::Process(x) => x,
            _ => panic!("calling with wrong object, expected process"),
        }
    }

    fn get_output(self) -> Result<Output, std::io::Error> {
        match self {
            OutputResult::Output(x) => x,
            _ => panic!("calling with wrong object, expected output"),
        }
    }
}

fn get_output(cmd: &mut Record, wait: bool) -> OutputResult {
    let field = cmd.native_field.as_mut().unwrap();
    let mut p = field
        .downcast_mut::<Command>()
        .unwrap()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(val) = cmd.get(&"input_buffer".to_string()) {
        match val {
            Value::Str(s) => {
                p.stdin
                    .as_mut()
                    .unwrap()
                    .write_all(s.as_ref().as_bytes())
                    .unwrap();
            }
            _ => unimplemented!(),
        }
    }
    if wait {
        OutputResult::Output(p.wait_with_output())
    } else {
        OutputResult::Process(p)
    }
}

// impls
#[hana_function]
fn out(mut cmd: Value::Record) -> Value {
    // stdout as string
    let out = get_output(cmd.inner_mut_ptr(), true).get_output().unwrap();
    match String::from_utf8(out.stdout) {
        Ok(s) => Value::Str((*vm).borrow().malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
}

#[hana_function]
fn err(mut cmd: Value::Record) -> Value {
    // stderr as string
    let out = get_output(cmd.inner_mut_ptr(), true).get_output().unwrap();
    match String::from_utf8(out.stderr) {
        Ok(s) => Value::Str((*vm).borrow().malloc(s.into())),
        Err(err) => {
            hana_raise!(vm, utf8_decoding_error(err, Rc::clone(&vm)));
        }
    }
}

#[hana_function]
fn outputs(mut cmd: Value::Record) -> Value {
    // array of [stdout, stderr] outputs
    let out = get_output(cmd.inner_mut_ptr(), true).get_output().unwrap();
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

// spawn
#[hana_function]
fn spawn(mut cmd: Value::Record) -> Value {
    let p = get_output(cmd.inner_mut_ptr(), false).get_process();
    let mut prec = (*vm).borrow().malloc(Record::new());
    prec.inner_mut_ptr().native_field = Some(Box::new(p));
    prec.inner_mut_ptr().insert(
        "prototype",
        Value::Record((*vm).borrow().stdlib.as_ref().unwrap().proc_rec.clone()),
    );
    Value::Record(prec)
}
