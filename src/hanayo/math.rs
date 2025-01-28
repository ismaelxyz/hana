//! Provides built-in math functions
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;

#[hana_function()]
fn sqrt(val: Value::Float) -> Value {
    Value::Float(val.sqrt())
}
