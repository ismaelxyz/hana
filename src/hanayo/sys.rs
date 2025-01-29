//! Provides Sys record
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;

#[hana_function()]
fn args() -> Value {
    let mut array = (*vm).borrow().malloc(Vec::new());
    for arg in std::env::args().skip(1) {
        array
            .inner_mut_ptr()
            .push(Value::Str((*vm).borrow().malloc(arg.to_string().into())));
    }
    Value::Array(array)
}
