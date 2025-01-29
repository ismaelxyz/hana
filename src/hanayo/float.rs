//! Provides Float record for handling floating point numbers
use std::str::FromStr;

use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;
use crate::harumachine::vmerror::VmError;

#[hana_function]
fn constructor(val: Value::Any) -> Value {
    match val {
        Value::Int(n) => Value::Float(n as f64),
        Value::Float(n) => Value::Float(n),
        Value::Str(s) => match f64::from_str(s.as_ref()) {
            Ok(n) => Value::Float(n),
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
                                .malloc("Can't convert string to float".to_string().into()),
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
                            .malloc("Can't convert value to float".to_string().into()),
                    ),
                );
                rec.inner_mut_ptr().insert("where", Value::Int(0));
                Value::Record(rec)
            });
        }
    }
}
