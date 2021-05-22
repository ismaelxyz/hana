//! Provides eval function for dynamically evaluating source code
use crate::compiler::{Compiler, ModulesInfo};
#[allow(unused_imports)]
use crate::vmbindings::interned_string_map::InternedStringMap;
use crate::vmbindings::value::Value;
use crate::vmbindings::vm::Vm;
use crate::vmbindings::vm::VmOpcode;
use std::cell::RefCell;
use std::rc::Rc;

#[hana_function()]
fn eval(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Ok(prog) = crate::grammar::parser_start(&s) {
        let target_ip = vm.code.len() as u32;
        let mut c = Compiler::new_append(
            vm.code.clone(),
            Rc::new(RefCell::new(ModulesInfo::new())),
            vm.interned_strings.take().unwrap(),
        );
        // generate code
        for stmt in prog {
            if stmt.emit(&mut c).is_err() {
                return Value::False;
            }
        }
        c.cpushop(VmOpcode::Halt);
        //panic!("{:?}", c.interned_strings);
        vm.interned_strings = c.interned_strings.take();
        vm.code = c.into_code();
        let ctx = vm.new_exec_ctx();
        vm.jmp(target_ip);
        vm.execute();
        vm.restore_exec_ctx(ctx);
        return Value::True;
    }
    Value::False
}
