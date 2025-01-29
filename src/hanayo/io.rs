//! Provides print, input and exit functions
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;

/// # Safety
///
/// This function needs to be unsafe for internal compatibility between multiple languages.
pub fn print(vm: Rc<RefCell<Vm>>, nargs: u16) {
    
    for _ in 0..nargs {
        let val = vm.borrow_mut().stack.pop().unwrap();
        std::print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
    vm.borrow_mut().stack.push(Value::Nil);
}

#[hana_function()]
fn input() -> Value {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer.pop(); // remove newline
    Value::Str((*vm).borrow().malloc(buffer.into()))
}

#[hana_function()]
fn exit(code: Value::Int) -> Value {
    std::process::exit(code as i32);
}
