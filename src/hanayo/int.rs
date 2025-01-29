//! Provides Int record for handling integers
use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;
use crate::harumachine::vmerror::VmError;
use std::str::FromStr;

#[hana_function]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Int(n),
        Value::Float(n) => Value::Int(n as i64),
        Value::Str(s) => match i64::from_str(s.as_ref()) {
            Ok(n) => Value::Int(n),
            Err(_) => {
                hana_raise!(vm, {
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
                            (*vm)
                                .borrow()
                                .malloc("Can't convert string to integer".to_string().into()),
                        ),
                    );
                    rec.inner_mut_ptr().insert("where", Value::Int(0));
                    Value::Record(rec)
                });
            }
        },
        _ => {
            hana_raise!(vm, {
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
                        (*vm)
                            .borrow()
                            .malloc("Can't convert value to integer".to_string().into()),
                    ),
                );
                rec.inner_mut_ptr().insert("where", Value::Int(0));
                Value::Record(rec)
            });
        }
    }
}

#[hana_function]
fn chr(i: Value::Int) -> Value {
    if let Some(ch) = std::char::from_u32(i as u32) {
        Value::Str((*vm).borrow().malloc(ch.to_string().into()))
    } else {
        Value::Nil
    }
}

#[hana_function]
fn hex(i: Value::Int) -> Value {
    Value::Str((*vm).borrow().malloc(format!("0x{:x}", i).into()))
}
