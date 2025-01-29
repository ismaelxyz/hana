//! Provides Time record for handling time
use crate::harumachine::record::Record;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;
use crate::harumachine::vmerror::VmError;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep as nsleep;
use std::time::*;

fn duration_to_record(vm: Rc<RefCell<Vm>>, duration: Duration) -> Value {
    let mut rec = (*vm).borrow().malloc(Record::new());
    rec.inner_mut_ptr().native_field = Some(Box::new(duration));
    rec.inner_mut_ptr().insert(
        "prototype",
        Value::Record((*vm).borrow().stdlib.as_ref().unwrap().time_rec.clone()),
    );
    Value::Record(rec)
}

#[hana_function()]
fn constructor() -> Value {
    duration_to_record(Rc::clone(&vm), SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
}

// since
#[hana_function()]
fn since(left: Value::Record, right: Value::Record) -> Value {
    let lfield = left.as_ref().native_field.as_ref().unwrap();
    let left_duration = *lfield.downcast_ref::<Duration>().unwrap();
    let rfield = right.as_ref().native_field.as_ref().unwrap();
    let right_duration = rfield.downcast_ref::<Duration>().unwrap();

    duration_to_record(vm, left_duration.checked_sub(*right_duration).unwrap())
}

// accessors
#[hana_function()]
fn secs(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_secs() as i64)
}
#[hana_function()]
fn millis(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_millis() as i64)
}
#[hana_function()]
fn micros(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_micros() as i64)
}
#[hana_function()]
fn nanos(time: Value::Record) -> Value {
    let tref = time.as_ref().native_field.as_ref().unwrap();
    let time = tref.downcast_ref::<Duration>().unwrap();
    Value::Int(time.as_nanos() as i64)
}

// other
#[hana_function()]
fn sleep(time: Value::Any) -> Value {
    match time {
        Value::Int(x) => {
            nsleep(Duration::from_secs(x as u64));
        }
        Value::Record(time) => {
            let tref = time.as_ref().native_field.as_ref().unwrap();
            let time = tref.downcast_ref::<Duration>().unwrap();
            nsleep(*time);
        }
        _ => {
            hana_raise!(vm, {
                let mut rec = (*vm).borrow().malloc(Record::new());
                rec.inner_mut_ptr().insert(
                    "prototype",
                    Value::Record((*vm).borrow().stdlib.as_ref().unwrap().invalid_argument_error.clone())
                        ,
                );
                rec.inner_mut_ptr().insert(
                    "why",
                    Value::Str(
                        (*vm).borrow().malloc(
                            "time must either be an Int or a Time record"
                                .to_string()
                                .into(),
                        ),
                    )
                    ,
                );
                rec.inner_mut_ptr().insert("where", Value::Int(0));
                Value::Record(rec)
            });
        }
    }
    Value::Nil
}
