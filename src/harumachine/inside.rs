use crate::harumachine::value::{value_is_true, Value::*};

#[allow(unused_imports)]
use crate::harumachine::{
    self,
    exframe::ExFrame,
    function::Function,
    gc::Gc,
    operations::*,
    //  nativeval::{NativeValue, NativeValueType::TYPE_INTERPRETER_ERROR},
    value::Value,
    vm::{Vm, VmOpcode, VmOpcode::*}, // vm_execute
    vmerror::VmError::{
        ERROR_CANNOT_ACCESS_NON_RECORD, ERROR_CASE_EXPECTS_DICT, ERROR_CONSTRUCTOR_NOT_FUNCTION,
        ERROR_EXPECTED_CALLABLE, ERROR_EXPECTED_ITERABLE, ERROR_EXPECTED_RECORD_ARRAY,
        ERROR_EXPECTED_RECORD_OF_EXPR, ERROR_KEY_NON_INT, ERROR_MISMATCH_ARGUMENTS,
        ERROR_NO_ERROR, ERROR_OP_ADD, ERROR_OP_BITWISE_AND, ERROR_OP_BITWISE_OR,
        ERROR_OP_BITWISE_XOR, ERROR_OP_DIV, ERROR_OP_EQ, ERROR_OP_GEQ, ERROR_OP_GT, ERROR_OP_LEQ,
        ERROR_OP_LT, ERROR_OP_MOD, ERROR_OP_MUL, ERROR_OP_NEQ, ERROR_OP_SUB,
        ERROR_RECORD_KEY_NON_STRING, ERROR_RECORD_NO_CONSTRUCTOR, ERROR_UNBOUNDED_ACCESS,
        ERROR_UNDEFINED_GLOBAL_VAR, ERROR_UNHANDLED_EXCEPTION, ERROR_UNKNOWN_KEY,
    },
};
use crate::harumachine::{/*env::Env */ record, string::HaruString};
use std::cell::RefCell;
use std::ops::Deref;
use std::{borrow::Borrow, rc::Rc};
use unicode_segmentation::UnicodeSegmentation;
// use crate::vmbindings::gc::Gc;

macro_rules! log_debug {
    () => (
        #[cfg(feature = "debuger")]
        $crate::print!("\n")
    );

    ($($arg:tt)*) => ({
        #[cfg(feature = "debuger")]
        println!($($arg)*);
    })
}

// Generate string fom bytecode.
// must be null terminated?
fn generate_string(vm: Rc<RefCell<Vm>>) -> String {
    log_debug!("  GenerateString(Start): {}", (*vm).borrow().ip);
    vm.borrow_mut().ip += 2;

    let mut key = Vec::new();
    {
        let vm_borrow = (*vm).borrow();
        let code_slice = &vm_borrow.code[vm_borrow.ip as usize - 1..];
        for &c in code_slice.iter() {
            if c == 0u8 {
                break;
            }
            key.push(c);
        }
    }

    vm.borrow_mut().ip += key.len() as u32;

    log_debug!("  GenerateString(End): {}", (*vm).borrow().ip);
    String::from_utf8(key).unwrap()
}

#[inline(always)]
pub(crate) fn get_prototype(vm: Rc<RefCell<Vm>>, val: Value) -> Option<Gc<record::Record>> {
    match val {
        Str(..) => (*vm).borrow().dstr.clone(),
        Int(..) => (*vm).borrow().dint.clone(),
        Float(..) => (*vm).borrow().dfloat.clone(),
        Array(..) => (*vm).borrow().darray.clone(),
        Record(reco) => {
            let reco = unsafe { &*reco.to_raw() };
            let proto = reco.get("prototype");
            if proto.is_none() {
                None
            } else if let Record(proto) = proto.unwrap() {
                Some(proto.clone())
            } else {
                unreachable!();
            }
        }
        _ => None,
    }
}

// pops a function/record constructor on top of the stack,
// sets up necessary environment and calls it.
#[inline(always)]
pub(super) fn inside_execute(vm: Rc<RefCell<Vm>>) {
    // println!("Ip: {}, {:?}", vm.ip, (*vm).borrow().code, vm.stack);
    /*
        vm.code: Instructions in the form of Bytecode
        vm.ip: Global Parting with reference to vm.code (Current Instruction).
        vm.stack: Stack/Saving of temporary values
    */

    //println!("Call me: {:?},  vm.code: {}", vm.code, vm.ip);

    if Halt == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Halt, IP: {}", (*vm).borrow().ip);
        return;
    }

    if Push8 == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        vm.borrow_mut().ip += 2;
        log_debug!("Push8, IP: {} sum(2)", (*vm).borrow().ip);

        let int = Int((*vm).borrow().code[(*vm).borrow().ip as usize - 1] as i64);
        log_debug!("  int: {:?}", &int);

        vm.borrow_mut().stack.push(int);
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    if Push16 == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Push16, IP: {}", (*vm).borrow().ip);
        #[rustfmt::skip]
        let i = i64::from_be_bytes([
            0,0,0,0,0,0,
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);

        vm.borrow_mut().stack.push(Int(i));
        vm.borrow_mut().ip += 3;
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    if Push32 == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Push32, IP: {}", (*vm).borrow().ip);
        #[rustfmt::skip]
        let i = i64::from_be_bytes([
            0, 0, 0, 0,
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 4],
        ]);

        vm.borrow_mut().stack.push(Int(i));
        vm.borrow_mut().ip += 5;
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    if Push64 == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Push64, IP: {}", (*vm).borrow().ip);
        let i = i64::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 4],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 5],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 6],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 7],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 8],
        ]);

        vm.borrow_mut().stack.push(Int(i));
        vm.borrow_mut().ip += 9;
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }
    // Push 32/64-bit float on to the stack
    if Pushf64 == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Pushf64, IP: {}", (*vm).borrow().ip);

        let bytes = [
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 4],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 5],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 6],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 7],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 8],
        ];
        vm.borrow_mut().ip += 9;
        let f = f64::from_bits(u64::from_ne_bytes(bytes));
        let float = Float(f);
        vm.borrow_mut().stack.push(float);
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    if PushBool == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        unimplemented!("This is for the future");
    }

    // Push string on to the stack
    if PushStr == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("PushStr, IP: {}", (*vm).borrow().ip);
        let key: HaruString = generate_string(Rc::clone(&vm)).into();
        log_debug!("  key: {:?}", key.to_string());
        let key = (*vm).borrow().malloc(key);
        vm.borrow_mut().stack.push(Str(key));

        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    // Get a store string and push on the stack
    if PushStrInterned == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("PushStrInterned, IP: {}", (*vm).borrow().ip);

        let i = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);

        vm.borrow_mut().ip += 3;
        let i = unsafe { (*vm).borrow().get_interned_string(i) };
        vm.borrow_mut().stack.push(Str((*vm).borrow().malloc(i)));
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    // Push nil on the stack
    if PushNil == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("PushNil, IP: {}", (*vm).borrow().ip);
        // log!("Push Nil");
        vm.borrow_mut().ip += 1;
        vm.borrow_mut().stack.push(Nil);
        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    // frees top of the stack and pops the stack
    if Pop == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        vm.borrow_mut().ip += 1;
        let _value = vm.borrow_mut().stack.pop();

        log_debug!(
            "Pop, IP: {} sum(1)\n  value: {:?}",
            (*vm).borrow().ip,
            _value
        );

        debug_assert!((*vm).borrow().ip as usize <= (*vm).borrow().code.len());
    }

    // pops top of the stack, performs unary not and pushes the result
    if Not == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Not, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        let val = vm.borrow_mut().stack.pop().unwrap();
        vm.borrow_mut().stack.push(Int(!value_is_true(val) as i64));
    }

    // pops top of the stack, performs unary negation and pushes the result
    if Negate == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Negate, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;

        let val = vm.borrow_mut().stack.pop().unwrap();
        match val {
            Int(i) => vm.borrow_mut().stack.push(Int(-i)),
            Float(f) => vm.borrow_mut().stack.push(Float(-f)),
            _ => unreachable!(""),
        }
    }

    // NOTE(xyz): IADD seems to me to be just for checking but it could have another
    // purpose. IADD as u8 {
    if Add == (*vm).borrow().code[(*vm).borrow().ip as usize]
        || IAdd == (*vm).borrow().code[(*vm).borrow().ip as usize]
    {
        log_debug!("ADD/IADD, IP: {}", (*vm).borrow().ip);
        let op = (*vm).borrow().code[(*vm).borrow().ip as usize];
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_add(left.clone(), right.clone(), Rc::clone(&vm));
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_ADD;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;

            return;
        }

        if IAdd == op {
            // Do task IADD...
        }

        vm.borrow_mut().stack.push(result);
    }

    if Sub == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Sub, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_sub(left.clone(), right.clone(), Rc::clone(&vm));
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_SUB;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if Mul == (*vm).borrow().code[(*vm).borrow().ip as usize]
        || IMul == (*vm).borrow().code[(*vm).borrow().ip as usize]
    {
        log_debug!("MUL/IMUL, IP: {}", (*vm).borrow().ip);
        let op = (*vm).borrow().code[(*vm).borrow().ip as usize];
        vm.borrow_mut().ip += 1;
        debug_assert!(vm.borrow_mut().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_mul(left.clone(), right.clone(), Rc::clone(&vm));
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_MUL;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        if IMul == op {
            // Error: ERROR_OP_IMUL...
            // Do task MUL...
        }
        vm.borrow_mut().stack.push(result);
    }

    if Div == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("DIV, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_div(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_DIV;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }
        vm.borrow_mut().stack.push(result);
    }

    if Mod == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("MOD, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_mod(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_MOD;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if BitwiseAnd == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("BITWISE_AND, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_bitwise_and(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_BITWISE_AND;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if BitwiseOr == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("BITWISE_OR, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_bitwise_or(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_BITWISE_OR;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if BitwiseXOR == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("BITWISE_XOR, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_bitwise_xor(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_BITWISE_XOR;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if Lt == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("LT, IP: {}", (*vm).borrow().ip);

        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_lt(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_LT;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }
    if LEq == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("LEQ, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_leq(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_LEQ;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if Gt == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Gt, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_gt(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_GT;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if GEq == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("GEq, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_geq(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_GEQ;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if Eq == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Eq, IP: {}", (*vm).borrow().ip);
        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_eq(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_EQ;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    if NEq == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("NEq, IP: {}", (*vm).borrow().ip);

        vm.borrow_mut().ip += 1;
        debug_assert!((*vm).borrow().stack.len() >= 2);
        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        let result = value_neq(left.clone(), right.clone());
        if let Value::InterpreterError = result {
            vm.borrow_mut().stack.append(&mut vec![right, left]); // tmp
            vm.borrow_mut().error = ERROR_OP_NEQ;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
            return;
        }

        vm.borrow_mut().stack.push(result);
    }

    // matching (these require the stdlib to be loaded)
    if Of == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Of, IP: {}", (*vm).borrow().ip);
        //LOG("OF\n");
        debug_assert!((*vm).borrow().stack.len() >= 2);
        vm.borrow_mut().ip += 1;

        let right = vm.borrow_mut().stack.pop().unwrap();
        let left = vm.borrow_mut().stack.pop().unwrap();

        unsafe {
            if let Record(rhs) = right {
                let rhs = rhs.to_raw();

                if let Record(proto) = left {
                    if rhs == (*vm).borrow().drec.as_ref().unwrap().to_raw() {
                        vm.borrow_mut().stack.push(Int(1));
                    } else {
                        let proto = &*proto.into_raw();
                        vm.borrow_mut()
                            .stack
                            .push(Int(proto.is_prototype_of(&*rhs) as i64));
                    }
                } else if let Some(true) =
                    get_prototype(Rc::clone(&vm), left).map(|reco| reco.to_raw() == rhs)
                {
                    vm.borrow_mut().stack.push(Int(1));
                } else {
                    vm.borrow_mut().stack.push(Int(0));
                }
            } else {
                vm.borrow_mut().error = ERROR_EXPECTED_RECORD_OF_EXPR;
                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                return;
            }
        }
    }

    // variables
    // creates a new environment whenever a function is called
    // the environment is initialized with a copy of the current environment's
    if EnvNew == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("EnvNew, Ip: {} sum(3)", vm.ip);
        let nslots = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        vm.borrow_mut().ip += 3;

        unsafe {
            let mut vm_mut = vm.borrow_mut();

            if let Some(last_entry) = vm_mut.localenv.last_mut() {
                let mut env = last_entry.borrow_mut().take().unwrap();
                env.reserve(nslots);
                debug_assert!(vm_mut.stack.len() >= env.nargs as usize);

                log_debug!("  send ars: {}", env.nargs);

                for i in 0..env.nargs {
                    let val = vm_mut.stack.pop().expect("the bytecode interpreter does not have all the arguments that the user requires.");

                    env.set(i, val);
                }

                *vm_mut.localenv.last_mut().unwrap().borrow_mut() = Some(env);
            }
        }
    }

    // variables
    // sets the value of current environment's slot to the top of the stack
    if SetLocal == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("SetLocal, IP: {}", vm.ip);
        let nslots = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        vm.borrow_mut().ip += 3;

        unsafe {
            if let Some(last_entry) = vm.borrow_mut().localenv.last_mut() {
                if let Some(env) = (**last_entry).borrow_mut().as_mut() {
                    let new_value = (*vm).borrow().stack[(*vm).borrow().stack.len() - 1].clone();
                    env.set(nslots, new_value);
                }
            }
        }
    }

    // this is for recursive function
    if SetLocalFunctionDef == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("SetLocalFunctionDef, IP: {}", vm.ip);
        let nslots = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        vm.borrow_mut().ip += 3;

        unsafe {
            if let Some(last_entry) = vm.borrow_mut().localenv.last_mut() {
                if let Some(env) = (**last_entry).borrow_mut().as_mut() {
                    env.set(
                        nslots,
                        (*vm).borrow().stack[(*vm).borrow().stack.len() - 1].clone(),
                    );

                    if let Fn(ref fun) = (*vm).borrow().stack[(*vm).borrow().stack.len() - 1] {
                        if let Some(bound) = fun.as_ref().bound.borrow_mut().as_mut() {
                            bound.set(
                                nslots,
                                (*vm).borrow().stack[(*vm).borrow().stack.len() - 1].clone(),
                            );
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }

    // pushes a copy of the value of current environment's slot
    if GetLocal == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        let slot = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        let mut vm_mut = vm.borrow_mut();
        vm_mut.ip += 3;
        log_debug!("GetLocal, IP: {} sum(3), slot {}", vm.ip, slot);
        let mut env = None;
        unsafe {
            if let Some(last_entry) = vm_mut.localenv.last_mut() {
                env = last_entry.borrow_mut().take();
                if let Some(env) = &mut env {
                    let value = env.get(slot);
                    log_debug!("  value: {:?}", &value);

                    vm_mut.stack.push(value);
                }
            }

            if let Some(last_entry) = vm_mut.localenv.last_mut() {
                *last_entry.borrow_mut() = env;
            }
        }
    }

    if GetLocalUp == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("GetLocalUp, IP: {}", vm.ip);
        let slot = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);

        let relascope = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 4],
        ]);
        vm.borrow_mut().ip += 5;

        unsafe {
            if let Some(last_entry) = vm.borrow_mut().localenv.last_mut() {
                if let Some(env) = (**last_entry).borrow().as_ref() {
                    vm.borrow_mut().stack.push(env.get_up(relascope, slot));
                }
            }
        }
    }

    // sets the value of the global variable to the top of the stack
    if SetGlobal == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("SetGlobal, IP: {}", vm.ip);
        let key = generate_string(Rc::clone(&vm));
        log_debug!("  key: {}", &key);
        let obj = (*vm).borrow().stack[(*vm).borrow().stack.len() - 1].clone();
        vm.borrow_mut().mut_global().insert(key.into(), obj);
    }

    // pushes a copy of the value of the global variable
    // WARNING: This condition may not act as expected?
    if GetGlobal == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("GetGlobal, IP: {}", vm.ip);
        let key = generate_string(Rc::clone(&vm));
        log_debug!("  key: {}", &key);

        let v = (*vm).borrow().global().get(&key).cloned();

        if let Some(val) = v {
            vm.borrow_mut().stack.push(val);
        } else {
            vm.borrow_mut().error = ERROR_UNDEFINED_GLOBAL_VAR;
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - (key.len() as i32 + 2)) as u32;
            log_debug!("  IP: {}", &vm.ip);
            return;
        }
    }

    // pushes a function with [name], that begins at the next instruction pointer
    // to the stack and jumps to the [end address]
    if DefFunctionPush == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("DefFunctionPush, IP: {}", vm.ip);
        // [opcode][end address]
        let nargs = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        let pos = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 4],
        ]);

        vm.borrow_mut().ip += 3;
        log_debug!("  args: {}, pos: {} sum(3)", nargs, pos);

        unsafe {
            let env = match (*vm).borrow().localenv.last() {
                Some(e) => Rc::clone(e),
                None => Rc::default(),
            };

            let ip = (*vm).borrow().ip + 2;
            let new_fn = Fn((*vm).borrow().malloc(Function::new(ip, nargs, env)));
            vm.borrow_mut().stack.push(new_fn);
        }

        vm.borrow_mut().ip += pos as u32;
    }

    // flow control
    // jmp [32-bit position] (jump to back)
    if Jmp == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("OP_JMP, IP: {}", vm.ip);
        vm.borrow_mut().ip += 1;
        let pos = i16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
        ]);

        let ip = ((*vm).borrow().ip as i32 + pos as i32) as u32;
        vm.borrow_mut().ip = ip;
    }

    // jmp [32-bit position]
    if JmpLong == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("OP_JMP_LONG, IP: {}", vm.ip);
        vm.borrow_mut().ip += 1;
        let pos = u32::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 3],
        ]);

        vm.borrow_mut().ip = pos;
    }

    // jmp if not true [32-bit position]
    if [JCond as u8, JCondNoPop as u8].contains(&(*vm).borrow().code[(*vm).borrow().ip as usize]) {
        log_debug!("OP_JCOND/OP_JCOND_NO_POP, IP: {}", vm.ip);

        let val = if JCond == (*vm).borrow().code[(*vm).borrow().ip as usize] {
            vm.borrow_mut().stack.pop().unwrap()
        } else {
            (*vm).borrow().stack.last().cloned().unwrap()
        };

        vm.borrow_mut().ip += 1;
        let pos = i16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
        ]);

        if value_is_true(val) {
            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 + pos as i32) as u32;
        } else {
            vm.borrow_mut().ip += 2;
        }
    }

    // jump (to back) if true [32-bit position]
    // Jump-Condition JMP
    if [JNcond as u8, JNcondNoPop as u8].contains(&(*vm).borrow().code[(*vm).borrow().ip as usize])
    {
        log_debug!("OP_JNCOND/OP_JNCOND_NO_POP");
        let val = if JNcond == (*vm).borrow().code[(*vm).borrow().ip as usize] {
            vm.borrow_mut().stack.pop().unwrap()
        } else {
            (*vm).borrow().stack.last().cloned().unwrap()
        };

        vm.borrow_mut().ip += 1;
        let pos = i16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
        ]);

        if !value_is_true(val) {
            let ip = ((*vm).borrow().ip as i32 + pos as i32) as u32;
            vm.borrow_mut().ip = ip;
        } else {
            vm.borrow_mut().ip += 2;
        }
    }

    if Call == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Call, IP: {}", (*vm).borrow().ip);

        let val = vm.borrow_mut().stack.pop().unwrap();
        let nargs = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);
        vm.borrow_mut().ip += 3;
        log_debug!("  sum(3), val: {}, nargs: {}", val, nargs);

        debug_assert!((*vm).borrow().stack.len() >= nargs as usize);
        match val {
            NativeFn(native) => {
                vm.borrow_mut().native_call_depth += 1;

                // Call to native function
                native(Rc::clone(&vm), nargs);

                vm.borrow_mut().native_call_depth -= 1;
                let call_depth = if (*vm).borrow().exframe_fallthrough.is_some() {
                    (*vm)
                        .borrow()
                        .exframe_fallthrough
                        .as_ref()
                        .unwrap()
                        .unwind_native_call_depth
                } else {
                    (*vm).borrow().native_call_depth
                };

                if call_depth != (*vm).borrow().native_call_depth
                    || (*vm).borrow().error != ERROR_NO_ERROR
                {
                    return;
                }
            }
            Record(ref reco) => {
                let pctor = unsafe { (*reco.to_raw()).get("new") };
                if pctor.is_none() {
                    vm.borrow_mut().error = ERROR_RECORD_NO_CONSTRUCTOR;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                    return;
                }
                let ctor = pctor.unwrap();
                match ctor {
                    NativeFn(native) => {
                        vm.borrow_mut().native_call_depth += 1;

                        // Call to native function
                        native(Rc::clone(&vm), nargs);

                        vm.borrow_mut().native_call_depth -= 1;
                        let call_depth = if (*vm).borrow().exframe_fallthrough.is_some() {
                            (*vm)
                                .borrow()
                                .exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                        } else {
                            (*vm).borrow().native_call_depth
                        };

                        if call_depth != (*vm).borrow().native_call_depth
                            || (*vm).borrow().error != ERROR_NO_ERROR
                        {
                            return;
                        }

                        if (*vm).borrow().exframe_fallthrough.is_some()
                            && (*vm)
                                .borrow()
                                .exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                                != (*vm).borrow().native_call_depth
                        {
                            return;
                        }
                    }
                    Fn(ifn) => {
                        let ifn = ifn.to_raw();
                        unsafe {
                            if nargs + 1 != (*ifn).nargs {
                                vm.borrow_mut().error = ERROR_MISMATCH_ARGUMENTS;
                                vm.borrow_mut().error_expected = (*ifn).nargs as u32;
                                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                                return;
                            }
                            let mut new_val = record::Record::new();
                            new_val.insert("prototype", val);
                            vm.borrow_mut()
                                .stack
                                .push(Record((*vm).borrow().malloc(new_val)));
                            vm.borrow_mut().enter_env(&*ifn);
                        }
                    }
                    _ => {
                        vm.borrow_mut().error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
                        vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                        return;
                    }
                }
            }
            Fn(hfn) => unsafe {
                let ifn = hfn.to_raw();
                if nargs != (*ifn).nargs {
                    vm.borrow_mut().error = ERROR_MISMATCH_ARGUMENTS;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                    vm.borrow_mut().error_expected = (*ifn).nargs as u32;
                    return;
                }
                vm.borrow_mut().enter_env(&*ifn);
                // vm.localenv();
            },
            _ => {
                vm.borrow_mut().error = ERROR_EXPECTED_CALLABLE;
                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                return;
            }
        }
    }

    // returns from function
    if Ret == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Ret, IP: {}", vm.ip);
        let last_entry = Rc::clone((*vm).borrow().localenv.last().unwrap());

        if let Some(env) = (*last_entry).borrow().as_ref() {
            if env.retip == u32::MAX {
                //LOG("return from vm_call\n");
                return;
            }
        }

        (*vm).borrow_mut().leave_env();

        //LOG("ip = %d\n", vm->ip);
    }

    // dictionaries
    if DictNew == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("OP_DICT_NEW");
        vm.borrow_mut().ip += 1;
        unimplemented!();
    }

    if MemberGet == (*vm).borrow().code[(*vm).borrow().ip as usize]
        || MemberGetNoPop == (*vm).borrow().code[(*vm).borrow().ip as usize]
    {
        log_debug!("MemberGet/MemberGetNoPop");
        let op = (*vm).borrow().code[(*vm).borrow().ip as usize];

        let val = (*vm).borrow().stack.last().cloned().unwrap();

        let pos = (*vm).borrow().ip;
        let key = generate_string(Rc::clone(&vm));

        let dict;
        if let Record(reco) = val {
            dict = Some(reco.clone());
            if op == MemberGet as u8 {
                vm.borrow_mut().stack.pop();
            }
        } else {
            dict = get_prototype(Rc::clone(&vm), val.clone());
            if dict.is_none() {
                vm.borrow_mut().error = ERROR_CANNOT_ACCESS_NON_RECORD;
                vm.borrow_mut().ip = (pos as i32 - 1) as u32;
                return;
            }

            if key == *"prototype" {
                if op == MemberGet as u8 {
                    vm.borrow_mut().stack.pop();
                }

                vm.borrow_mut().stack.push(Record(dict.clone().unwrap()));
                return; // Does this go here?
            }
        }

        let result = dict.unwrap().as_ref().get(&key).cloned();
        if let Some(result) = result {
            vm.borrow_mut().stack.push(result);
        } else {
            vm.borrow_mut().error = ERROR_UNKNOWN_KEY;
            vm.borrow_mut().ip = (pos as i32 - 1) as u32; // or? vm.borrow_mut().ip = pos;
            return;
        }
    }

    if MemberSet == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("MemberSet, IP: {}", vm.ip);
        // stack: [value][dict]
        let pos = (*vm).borrow().ip;
        let key = generate_string(Rc::clone(&vm));

        let dval = vm.borrow_mut().stack.last().cloned().unwrap();
        match dval {
            Record(mut reco) => {
                vm.borrow_mut().stack.pop();
                let val = vm.borrow_mut().stack.pop().unwrap();
                reco.inner_mut_ptr().insert(key, val);
            }
            _ => {
                vm.borrow_mut().error = ERROR_CANNOT_ACCESS_NON_RECORD;
                vm.borrow_mut().ip = pos;
                return;
            }
        }
    }

    if DictLoad == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("DictLoad, IP: {}", vm.ip);
        // stack: [nil][value][key]
        vm.borrow_mut().ip += 1;

        let mut length = {
            let val = vm.borrow_mut().stack.pop().unwrap();
            let Int(num) = val else {
                unreachable!("Expect integer, found {}", val.type_name());
            };

            num as usize
        };

        let mut dval = record::Record::with_capacity(length);

        while length > 0 {
            //debug_assert(key.type == TYPE_STR);
            // key
            let key = {
                let val = vm.borrow_mut().stack.pop().unwrap();
                let Str(s) = val else {
                    unreachable!("Expect string, found {}", val.type_name());
                };

                s
            };
            // val
            let val = vm.borrow_mut().stack.pop().unwrap();
            let key = unsafe { (*key.to_raw()).borrow() } as &String;
            dval.insert(key.clone(), val);

            length -= 1;
        }

        vm.borrow_mut()
            .stack
            .push(Record((*vm).borrow().malloc(dval)));
    }

    if ArrayLoad == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("ArrayLoad, IP: {}", vm.ip);
        vm.borrow_mut().ip += 1;

        let mut length = {
            let val = vm.borrow_mut().stack.pop().unwrap();
            let Int(num) = val else {
                unreachable!("Expect integer, found {}", val.type_name());
            };

            num as usize
        };

        if length == 0 {
            vm.borrow_mut()
                .stack
                .push(Array((*vm).borrow().malloc(Vec::new())));
        } else {
            let mut array = Vec::with_capacity(length);

            while length > 0 {
                array.insert(0, vm.borrow_mut().stack.pop().unwrap());
                length -= 1
            }
            vm.borrow_mut()
                .stack
                .push(Array((*vm).borrow().malloc(array)));
        }
    }

    // exceptions
    if Try == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Try, IP: {}", vm.ip);
        // stack: [nil][function][error type]
        //LOG("TRY\n");
        vm.borrow_mut().ip += 1;

        let frame: *mut _ = vm.borrow_mut().enter_exframe();
        let mut error;
        loop {
            unsafe {
                error = vm.borrow_mut().stack.last().cloned().unwrap();
                if let Nil = error {
                    break;
                }

                // error type
                if let Record(reco) = error {
                    vm.borrow_mut().stack.pop();
                    // val
                    let xfn = (*vm).borrow().stack.last().cloned().unwrap();
                    let xfn = {
                        let Fn(f) = xfn else {
                            unreachable!();
                        };
                        f.to_raw()
                    };
                    //debug_assert!(xfn.type == TYPE_FN);
                    vm.borrow_mut().stack.pop();
                    (*frame).set_handler(Some(reco.clone()), (*xfn).clone());
                } else {
                    vm.borrow_mut().error = ERROR_CASE_EXPECTS_DICT;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    return;
                }
            }
        }

        vm.borrow_mut().stack.pop(); // pop nil
    }

    if Raise == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Raise, IP: {}", vm.ip);
        if !harumachine::vm::raise(Rc::clone(&vm)) {
            vm.borrow_mut().error = ERROR_UNHANDLED_EXCEPTION;
            if (*vm).borrow().exframe_fallthrough.is_some()
                || (*vm).borrow().native_call_depth != 0
            {
                log_debug!(
                    "falling through pls wait ({})\n",
                    (*vm).borrow().native_call_depth
                );
                return;
            }
            return;
        }

        if (*vm).borrow().exframe_fallthrough.is_some() || (*vm).borrow().native_call_depth != 0 {
            log_debug!(
                "falling through pls wait ({})\n",
                (*vm).borrow().native_call_depth
            );
            return;
        }
    }

    if ExframeRet == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("ExframeRet, IP: {}", vm.ip);
        vm.borrow_mut().ip += 1;
        let pos = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
        ]);

        vm.borrow_mut().ip += pos as u32;
        vm.borrow_mut().leave_exframe();
    }

    if RetCall == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("RetCall, IP: {}", (*vm).borrow().ip);
        let val = (*vm).borrow().stack.last().cloned().unwrap();
        let nargs = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 2],
        ]);

        vm.borrow_mut().ip += 3;
        debug_assert!((*vm).borrow().stack.len() >= (nargs as usize));

        match val {
            NativeFn(native) => {
                vm.borrow_mut().stack.pop();
                vm.borrow_mut().native_call_depth += 1;

                // Call to native function
                native(Rc::clone(&vm), nargs);

                vm.borrow_mut().native_call_depth -= 1;
                let call_depth = if (*vm).borrow().exframe_fallthrough.is_some() {
                    (*vm)
                        .borrow()
                        .exframe_fallthrough
                        .as_ref()
                        .unwrap()
                        .unwind_native_call_depth
                } else {
                    (*vm).borrow().native_call_depth
                };

                if call_depth != (*vm).borrow().native_call_depth
                    || (*vm).borrow().error != ERROR_NO_ERROR
                {
                    return;
                }

                let last_entry = Rc::clone((*vm).borrow().localenv.last().unwrap());

                if let Some(env) = (*last_entry).borrow().as_ref() {
                    if env.retip == u32::MAX {
                        return;
                    }
                }

                vm.borrow_mut().leave_env();
            }
            Fn(hfn) => unsafe {
                vm.borrow_mut().stack.pop();
                let ifn = hfn.to_raw();
                if nargs != (*ifn).nargs {
                    vm.borrow_mut().error = ERROR_MISMATCH_ARGUMENTS;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                    vm.borrow_mut().error_expected = (*ifn).nargs as u32;
                    return;
                }

                vm.borrow_mut().enter_env_tail(&*ifn);
                // vm.localenv();
            },
            Record(ref reco) => {
                let pctor = unsafe { (*reco.to_raw()).get("constructor") };
                if pctor.is_none() {
                    vm.borrow_mut().error = ERROR_RECORD_NO_CONSTRUCTOR;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                    return;
                }
                let ctor = pctor.unwrap();
                match ctor {
                    NativeFn(native) => {
                        vm.borrow_mut().native_call_depth += 1;

                        // Call to native function
                        native(Rc::clone(&vm), nargs);

                        vm.borrow_mut().native_call_depth -= 1;
                        let call_depth = if (*vm).borrow().exframe_fallthrough.is_some() {
                            (*vm)
                                .borrow()
                                .exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                        } else {
                            (*vm).borrow().native_call_depth
                        };

                        if call_depth != (*vm).borrow().native_call_depth
                            || (*vm).borrow().error != ERROR_NO_ERROR
                        {
                            return;
                        }

                        if let Some(last_entry) = (*vm).borrow().localenv.last() {
                            if let Some(env) = (**last_entry).borrow().as_ref() {
                                if env.retip == u32::MAX {
                                    //LOG("return from vm_call\n");
                                    return;
                                }
                            }
                        }
                    }
                    Fn(ifn) => {
                        let ifn = ifn.to_raw();
                        unsafe {
                            if nargs + 1 != (*ifn).nargs {
                                vm.borrow_mut().error = ERROR_MISMATCH_ARGUMENTS;
                                vm.borrow_mut().error_expected = (*ifn).nargs as u32;
                                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                                return;
                            }
                            let mut new_val = record::Record::new();
                            new_val.insert("prototype", val.clone());
                            vm.borrow_mut()
                                .stack
                                .push(Record((*vm).borrow().malloc(new_val)));

                            vm.borrow_mut().enter_env_tail(&*ifn);
                            // vm.localenv();
                        }
                    }
                    _ => {
                        vm.borrow_mut().error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
                        vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                        return;
                    }
                }
            }
            _ => {
                vm.borrow_mut().error = ERROR_EXPECTED_CALLABLE;
                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32;
                return;
            }
        }
    }

    // Remember the -2
    if ForIn == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("ForIn, IP: {}", vm.ip);

        vm.borrow_mut().ip += 1;
        let pos = u16::from_be_bytes([
            (*vm).borrow().code[(*vm).borrow().ip as usize],
            (*vm).borrow().code[(*vm).borrow().ip as usize + 1],
        ]);
        // as usize + 1]);
        unsafe {
            //println!("{:?}, {}", (*vm).borrow().code, vm.ip);
            debug_assert!(!(*vm).borrow().stack.is_empty());

            let top = (*vm).borrow().stack.last().cloned().unwrap();

            match top {
                Str(xstr) => {
                    let mut vec = (*xstr.to_raw())
                        .deref()
                        .chars()
                        .map(|ch| Str((*vm).borrow().malloc(ch.to_string().into())))
                        .collect::<Vec<_>>();

                    vm.borrow_mut().stack.pop();
                    if vec.is_empty() {
                        // skip empty
                        vm.borrow_mut().ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                    } else {
                        let less = vec.remove(0);
                        vm.borrow_mut()
                            .stack
                            .push(Array((*vm).borrow().malloc(vec)));
                        vm.borrow_mut().stack.push(Iterator);
                        vm.borrow_mut().stack.push(less);
                    }
                }
                Array(array) => {
                    let array = array.to_raw();
                    if (*array).is_empty() {
                        // skip empty
                        vm.borrow_mut().ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                        vm.borrow_mut().stack.pop();
                    } else {
                        vm.borrow_mut().stack.pop();
                        let mut clone = (*array).clone();
                        let less = clone.remove(0);

                        vm.borrow_mut()
                            .stack
                            .push(Array((*vm).borrow().malloc(clone))); // De esta manera no consumiremos el original
                        vm.borrow_mut().stack.push(Iterator);
                        vm.borrow_mut().stack.push(less);
                    }
                }
                //TYPE_DICT
                // interation
                Iterator => {
                    // There must be at least two values on the stack!
                    debug_assert!((*vm).borrow().stack.len() >= 2);

                    let len_stack = (*vm).borrow().stack.len();
                    let iterator = (*vm).borrow().stack[len_stack - 2].clone();
                    match iterator {
                        Nil => {
                            vm.borrow_mut().ip += (pos as i32 - 2) as u32;
                        }

                        // NativeValue
                        Array(arr) => {
                            let arr = arr.into_raw();
                            if (*arr).is_empty() {
                                vm.borrow_mut().stack.pop(); /* iterator */
                                vm.borrow_mut().stack.pop(); /* array */
                                vm.borrow_mut().ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                            } else {
                                log_debug!("CONTINUE\n");
                                //vm.stack.pop(); /* old iterator */
                                //array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR,
                                // (void *)(idx + 1)));
                                // vm.borrow_mut().stack.push(Iterator);
                                vm.borrow_mut().stack.push((*arr).remove(0));
                            }
                        }

                        _ => {
                            vm.borrow_mut().error = ERROR_EXPECTED_ITERABLE;
                            // 1 + sizeof(pos) (where 1 is the operator)
                            vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 3) as u32; // R(siz)()?
                            return;
                        }
                    }
                }
                _ => {
                    log_debug!("NOT ITERABLE\n");
                    vm.borrow_mut().error = ERROR_EXPECTED_ITERABLE;
                    // -2 == - sizeof(pos)
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 2) as u32; // R(siz)()?
                    return;
                }
            }

            vm.borrow_mut().ip += 2; // Sera un descuido R(siz)()?
        }
    }

    if IndexGet == (*vm).borrow().code[(*vm).borrow().ip as usize]
        || MemberGetNoPop == (*vm).borrow().code[(*vm).borrow().ip as usize]
    {
        log_debug!("IndexGET/MemberGetNoPop, IP: {}", vm.ip);
        let index = vm.borrow_mut().stack.pop().unwrap();
        let dval = if IndexGet == (*vm).borrow().code[(*vm).borrow().ip as usize] {
            vm.borrow_mut().stack.pop().unwrap()
        } else {
            (*vm).borrow().stack.last().cloned().unwrap()
        };

        vm.borrow_mut().ip += 1;
        match dval {
            Array(array) => {
                let array = unsafe { &*array.to_raw() };
                let index = if let Int(num) = index {
                    if num < 0 {
                        ((array.len() as i64) + num) as usize
                    } else {
                        num as usize
                    }
                } else {
                    vm.borrow_mut().error = ERROR_KEY_NON_INT;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    return;
                };

                if index >= array.len() {
                    vm.borrow_mut().error = ERROR_UNBOUNDED_ACCESS;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    vm.borrow_mut().error_expected = (array.len()) as u32;
                    return;
                }

                vm.borrow_mut().stack.push(array[index].clone())
            }
            Str(xstr) => {
                let xstr = unsafe { &*xstr.to_raw() };
                let len = xstr.graphemes(true).count() as i64;

                let index = if let Int(num) = index {
                    if num < 0 {
                        //   -
                        (len + num) as usize
                    } else {
                        num as usize
                    }
                } else {
                    vm.borrow_mut().error = ERROR_KEY_NON_INT;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    return;
                };

                let left = if let Some(ch) = xstr.graphemes(true).nth(index) {
                    (*vm).borrow().malloc(ch.to_string().into())
                } else {
                    vm.borrow_mut().error = ERROR_UNBOUNDED_ACCESS;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    vm.borrow_mut().error_expected = len as u32;
                    return;
                };
                vm.borrow_mut().stack.push(Str(left));
            }
            _ => {
                vm.borrow_mut().error = ERROR_CANNOT_ACCESS_NON_RECORD;
                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                return;
            }
        }
    }

    if IndexSet == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("IndexSet, IP: {}", vm.ip);
        vm.borrow_mut().ip += 1;

        let index = vm.borrow_mut().stack.pop().unwrap();
        let dval = vm.borrow_mut().stack.pop().unwrap();
        let val = (*vm).borrow().stack.last().unwrap().clone();

        match dval {
            Array(array) => {
                // Note: i64::MAX < usize::MAX
                let array = unsafe { &mut *(array.to_raw() as *mut Vec<_>) };
                let index = if let Int(num) = index {
                    if num < 0 {
                        ((array.len() as i64) + num) as usize
                    } else {
                        num as usize
                    }
                } else {
                    vm.borrow_mut().error = ERROR_KEY_NON_INT;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    return;
                };

                if index >= array.len() {
                    vm.borrow_mut().error = ERROR_UNBOUNDED_ACCESS;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    vm.borrow_mut().error_expected = (array.len()) as u32;
                    return;
                }

                array[index] = val.clone();
            }
            // TODO: the Record's should be more like classes
            // than dictionaries, in the future, Hana will support dictionaries.
            // dictionaries, at the moment we will continue, with the old
            // traditions :(.
            Record(mut reco) => unsafe {
                let index = if let Str(s) = index {
                    (*s.into_raw()).clone()
                } else {
                    vm.borrow_mut().error = ERROR_RECORD_KEY_NON_STRING;
                    vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                    return;
                };

                reco.inner_mut_ptr().insert(index, val.clone());
            },
            _ => {
                vm.borrow_mut().error = ERROR_EXPECTED_RECORD_ARRAY;
                vm.borrow_mut().ip = ((*vm).borrow().ip as i32 - 1) as u32;
                return;
            }
        }
    }

    if Swap == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Swap, IP: {}", vm.ip);
        debug_assert!((*vm).borrow().stack.len() >= 2);
        vm.borrow_mut().ip += 1;
        let stack = &mut (*vm).borrow_mut().stack;
        let len = stack.len();
        stack.swap(len - 1, len - 2);
    }

    // modules
    if Use == (*vm).borrow().code[(*vm).borrow().ip as usize] {
        log_debug!("Use, IP: {}", vm.ip);
        let path = generate_string(Rc::clone(&vm));
        vm.borrow_mut().load_module(&path);
    }

    inside_execute(vm);
}

pub(super) fn vm_call(vm: Rc<RefCell<Vm>>, func: Value, args: &[Value]) -> Value {
    let ifn: *mut Function = std::ptr::null_mut();
    let nargs = args.len() as u16;

    if let NativeFn(value_fn) = func {
        let len = args.len() as isize - 1;
        for i in -len..1 {
            vm.borrow_mut().stack.push(args[(-i) as usize].clone());
        }

        // call fn
        value_fn(Rc::clone(&vm), nargs);

        return vm.borrow_mut().stack.pop().clone().unwrap();
    } else if let Record(pctor) = func.clone() {
        let pctor = unsafe { (*pctor.to_raw()).get("constructor") };

        if pctor.is_none() {
            vm.borrow_mut().error = ERROR_RECORD_NO_CONSTRUCTOR;
            return Value::InterpreterError;
        }

        if let NativeFn(value_fn) = pctor.unwrap() {
            let len = args.len() as isize - 1;
            for i in -len..1 {
                vm.borrow_mut().stack.push(args[(-i) as usize].clone());
            }

            // call fn
            value_fn(Rc::clone(&vm), nargs);

            return vm.borrow_mut().stack.pop().unwrap();
        } else if let Fn(f) = func {
            unsafe { *ifn = (*f.into_raw()).clone() };
        } else {
            vm.borrow_mut().error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
            return Value::InterpreterError;
        }
    } else if let Fn(f) = func {
        unsafe { *ifn = (*f.into_raw()).clone() };
    } else {
        vm.borrow_mut().error = ERROR_EXPECTED_CALLABLE;
        return Value::InterpreterError;
    }
    unsafe {
        if nargs != (*ifn).nargs {
            vm.borrow_mut().ip = (*ifn).ip;
            vm.borrow_mut().error = ERROR_MISMATCH_ARGUMENTS;
            return Value::InterpreterError;
        }
    }

    let last = (*vm).borrow().ip;
    // setup env
    let oldenv = (*vm).borrow().localenv.len();
    vm.borrow_mut().ip = (last as i32 - 1) as u32;
    unsafe {
        vm.borrow_mut().enter_env(&*ifn);
    }
    let curenv = (*vm).borrow().localenv.len();
    // setup stack/ip

    let len = args.len() as isize - 1;
    for i in -len..1 {
        vm.borrow_mut().stack.push(args[(-i) as usize].clone());
    }

    inside_execute(Rc::clone(&vm));
    if (*vm).borrow().error != ERROR_NO_ERROR || (*vm).borrow().exframe_fallthrough.is_some() {
        // exception
        //LOG("falling through psl wait %ld", vm->native_call_depth);
        return Value::InterpreterError;
    }

    if (*vm).borrow().localenv.len() != curenv {
        // exception occurred outside of function's scope
        // NOTE: curenv already free'd from unwinding
        return Value::InterpreterError;
    }

    let localenv = vm.borrow_mut().localenv.drain(..oldenv).collect();
    vm.borrow_mut().localenv = localenv;
    vm.borrow_mut().ip = last;

    vm.borrow_mut().stack.pop().unwrap()
}
