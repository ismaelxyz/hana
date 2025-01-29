//! Provides eval function for dynamically evaluating source code
use crate::compiler::{Compiler, ModulesInfo};
#[allow(unused_imports)]
use crate::harumachine::interned_string_map::InternedStringMap;
use crate::harumachine::value::Value;
use crate::harumachine::vm::VmOpcode;
use crate::harumachine::vm::{execute_vm, Vm};
use std::cell::RefCell;
use std::rc::Rc;

#[hana_function]
fn eval(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Ok(prog) = crate::grammar::parser_start(s) {
        let target_ip = (*vm).borrow().code.len() as u32;
        let mut c = Compiler::new_append(
            (*vm).borrow().code.clone(),
            Rc::new(RefCell::new(ModulesInfo::new())),
            vm.borrow_mut().interned_strings.take().unwrap(),
        );
        // generate code
        for stmt in prog {
            if stmt.emit(&mut c).is_err() {
                return Value::False;
            }
        }

        c.cpushop(VmOpcode::Halt);
        //panic!("{:?}", c.interned_strings);
        vm.borrow_mut().interned_strings = c.interned_strings.take();
        vm.borrow_mut().code = c.into_code();
        let ctx = vm.borrow_mut().new_exec_ctx();
        vm.borrow_mut().jmp(target_ip);
        execute_vm(Rc::clone(&vm));
        vm.borrow_mut().restore_exec_ctx(ctx);
        return Value::True;
    }
    Value::False
}
