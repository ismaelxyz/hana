#[allow(unused_imports)]
use crate::vmbindings::value::{value_is_true, Value::*};
#[allow(unused_imports)]
use crate::vmbindings::{
    exframe::ExFrame,
    function::Function,
    gc::Gc,
    nativeval::{NativeValue, NativeValueType::TYPE_INTERPRETER_ERROR},
    operations::*,
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
use crate::vmbindings::{/*env::Env */ record, string::HaruString};
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
fn generate_string(vm: &mut Vm) -> String {
    log_debug!("  GenerateString(Start): {}", vm.ip);
    vm.ip += 2;
    let mut key = Vec::new();
    for c in &vm.code[vm.ip as usize - 1..] {
        if c == &0u8 {
            break;
        }
        key.push(*c);
        vm.ip += 1;
    }

    log_debug!("  GenerateString(End): {}", vm.ip);
    String::from_utf8(key).unwrap()
}

#[inline(always)]
pub(crate) fn get_prototype(vm: &Vm, val: NativeValue) -> *const record::Record {
    match unsafe { val.unwrap() } {
        Str(..) => vm.dstr.as_ref().unwrap().to_raw(),
        Int(..) => vm.dint.as_ref().unwrap().to_raw(),
        Float(..) => vm.dfloat.as_ref().unwrap().to_raw(),
        Array(..) => vm.darray.as_ref().unwrap().to_raw(),
        Record(reco) => {
            let reco = unsafe { &*reco.to_raw() };
            let proto = reco.get("prototype");
            if proto.is_none() {
                std::ptr::null()
            } else if let Record(proto) = unsafe { proto.unwrap().unwrap() } {
                proto.to_raw()
            } else {
                unreachable!();
            }
        }
        _ => std::ptr::null(),
    }
}

// pops a function/record constructor on top of the stack,
// sets up necessary environment and calls it.
#[inline(always)]
pub(super) fn inside_execute(vm: &mut Vm) {
    // println!("Ip: {}, {:?}", vm.ip, vm.code, vm.stack);
    /*
        vm.code: Instructions in the form of Bytecode
        vm.ip: Global Parting with reference to vm.code (Current Instruction).
        vm.stack: Stack/Saving of temporary values
    */

    //println!("Call me: {:?},  vm.code: {}", vm.code, vm.ip);

    if Halt == vm.code[vm.ip as usize] {
        log_debug!("Halt, IP: {}", vm.ip);
        return;
    }

    if Push8 == vm.code[vm.ip as usize] {
        vm.ip += 2;
        log_debug!("Push8, IP: {} sum(2)", vm.ip);

        let int = Int(vm.code[vm.ip as usize - 1] as i64).wrap();
        log_debug!("  int: {:?}", &int);

        vm.stack.push(int);
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    if Push16 == vm.code[vm.ip as usize] {
        log_debug!("Push16, IP: {}", vm.ip);
        #[rustfmt::skip]
        let i = i64::from_be_bytes([
            0,0,0,0,0,0,
            vm.code[vm.ip as usize + 1],
            vm.code[vm.ip as usize + 2],
        ]);
        vm.stack.push(Int(i).wrap());
        vm.ip += 3;
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    if Push32 == vm.code[vm.ip as usize] {
        log_debug!("Push32, IP: {}", vm.ip);
        #[rustfmt::skip]
        let i = i64::from_be_bytes([
            0, 0, 0, 0,
            vm.code[vm.ip as usize + 1],
            vm.code[vm.ip as usize + 2],
            vm.code[vm.ip as usize + 3],
            vm.code[vm.ip as usize + 4],
        ]);
        vm.stack.push(Int(i).wrap());
        vm.ip += 5;
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    if Push64 == vm.code[vm.ip as usize] {
        log_debug!("Push64, IP: {}", vm.ip);
        let i = i64::from_be_bytes([
            vm.code[vm.ip as usize + 1],
            vm.code[vm.ip as usize + 2],
            vm.code[vm.ip as usize + 3],
            vm.code[vm.ip as usize + 4],
            vm.code[vm.ip as usize + 5],
            vm.code[vm.ip as usize + 6],
            vm.code[vm.ip as usize + 7],
            vm.code[vm.ip as usize + 8],
        ]);
        vm.stack.push(Int(i).wrap());
        vm.ip += 9;
        debug_assert!(vm.ip as usize <= vm.code.len());
    }
    // Push 32/64-bit float on to the stack
    if Pushf64 == vm.code[vm.ip as usize] {
        log_debug!("Pushf64, IP: {}", vm.ip);

        let bytes = [
            vm.code[vm.ip as usize + 1],
            vm.code[vm.ip as usize + 2],
            vm.code[vm.ip as usize + 3],
            vm.code[vm.ip as usize + 4],
            vm.code[vm.ip as usize + 5],
            vm.code[vm.ip as usize + 6],
            vm.code[vm.ip as usize + 7],
            vm.code[vm.ip as usize + 8],
        ];
        vm.ip += 9;
        let f = f64::from_bits(u64::from_ne_bytes(bytes));
        let float = Float(f).wrap();
        vm.stack.push(float);
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    if PushBool == vm.code[vm.ip as usize] {
        unimplemented!("This is for the future");
    }

    // Push string on to the stack
    if PushStr == vm.code[vm.ip as usize] {
        log_debug!("PushStr, IP: {}", vm.ip);
        let key: HaruString = generate_string(vm).into();
        log_debug!("  key: {:?}", key.to_string());
        let key = vm.malloc(key);
        vm.stack.push(Str(key).wrap());

        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    // Get a store string and push on the stack
    if PushStrInterned == vm.code[vm.ip as usize] {
        log_debug!("PushStrInterned, IP: {}", vm.ip);

        let i = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        let i = unsafe { vm.get_interned_string(i) };
        vm.stack.push(Str(vm.malloc(i)).wrap());
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    // Push nil on the stack
    if PushNil == vm.code[vm.ip as usize] {
        log_debug!("PushNil, IP: {}", vm.ip);
        // log!("Push Nil");
        vm.ip += 1;
        vm.stack.push(Nil.wrap());
        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    // frees top of the stack and pops the stack
    if Pop == vm.code[vm.ip as usize] {
        vm.ip += 1;
        let _value = vm.stack.pop();

        log_debug!("Pop, IP: {} sum(1)\n  value: {:?}", vm.ip, _value);

        debug_assert!(vm.ip as usize <= vm.code.len());
    }

    // pops top of the stack, performs unary not and pushes the result
    if Not == vm.code[vm.ip as usize] {
        log_debug!("Not, IP: {}", vm.ip);
        vm.ip += 1;
        let val = vm.stack.pop().unwrap();
        vm.stack.push(Int(!value_is_true(val) as i64).wrap());
    }

    // pops top of the stack, performs unary negation and pushes the result
    if Negate == vm.code[vm.ip as usize] {
        log_debug!("Negate, IP: {}", vm.ip);
        vm.ip += 1;
        let val = vm.stack.pop().unwrap();
        match unsafe { val.unwrap() } {
            Int(i) => vm.stack.push(Int(-i).wrap()),
            Float(f) => vm.stack.push(Float(-f).wrap()),
            _ => unreachable!(""),
        }
    }

    // NOTE(xyz): IADD seems to me to be just for checking but it could have another
    // purpose. IADD as u8 {
    if Add == vm.code[vm.ip as usize] || IAdd == vm.code[vm.ip as usize] {
        log_debug!("ADD/IADD, IP: {}", vm.ip);
        let op = vm.code[vm.ip as usize];
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_add(left.clone().unwrap(), right.clone().unwrap(), vm) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_ADD;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }

        if IAdd == op {
            // Do task IADD...
        }

        vm.stack.push(result);
    }

    if Sub == vm.code[vm.ip as usize] {
        log_debug!("Sub, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_sub(left.clone().unwrap(), right.clone().unwrap(), vm) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_SUB;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if Mul == vm.code[vm.ip as usize] || IMul == vm.code[vm.ip as usize] {
        log_debug!("MUL/IMUL, IP: {}", vm.ip);
        let op = vm.code[vm.ip as usize];
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_mul(left.clone().unwrap(), right.clone().unwrap(), vm) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_MUL;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }

        if IMul == op {
            // Error: ERROR_OP_IMUL...
            // Do task MUL...
        }
        vm.stack.push(result);
    }

    if Div == vm.code[vm.ip as usize] {
        log_debug!("DIV, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_div(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_DIV;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if Mod == vm.code[vm.ip as usize] {
        log_debug!("MOD, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_mod(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_MOD;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if BitwiseAnd == vm.code[vm.ip as usize] {
        log_debug!("BITWISE_AND, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_bitwise_and(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_BITWISE_AND;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if BitwiseOr == vm.code[vm.ip as usize] {
        log_debug!("BITWISE_OR, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_bitwise_or(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_BITWISE_OR;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if BitwiseXOR == vm.code[vm.ip as usize] {
        log_debug!("BITWISE_XOR, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_bitwise_xor(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_BITWISE_XOR;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if Lt == vm.code[vm.ip as usize] {
        log_debug!("LT, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_lt(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_LT;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }
    if LEq == vm.code[vm.ip as usize] {
        log_debug!("LEQ, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_leq(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_LEQ;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }
    if Gt == vm.code[vm.ip as usize] {
        log_debug!("Gt, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_gt(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_GT;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }
    if GEq == vm.code[vm.ip as usize] {
        log_debug!("GEq, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_geq(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_GEQ;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }
    if Eq == vm.code[vm.ip as usize] {
        log_debug!("Eq, IP: {}", vm.ip);
        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();
        let result = unsafe { value_eq(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_EQ;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    if NEq == vm.code[vm.ip as usize] {
        log_debug!("NEq, IP: {}", vm.ip);

        vm.ip += 1;
        debug_assert!(vm.stack.len() >= 2);
        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();

        let result = unsafe { value_neq(left.clone().unwrap(), right.clone().unwrap()) };
        if result.r#type == TYPE_INTERPRETER_ERROR {
            vm.stack.append(&mut vec![right, left]); // tmp
            vm.error = ERROR_OP_NEQ;
            vm.ip = (vm.ip as i32 - 1) as u32;
            return;
        }
        vm.stack.push(result);
    }

    // matching (these require the stdlib to be loaded)
    if Of == vm.code[vm.ip as usize] {
        log_debug!("Of, IP: {}", vm.ip);
        //LOG("OF\n");
        debug_assert!(vm.stack.len() >= 2);
        vm.ip += 1;

        let right = vm.stack.pop().unwrap();
        let left = vm.stack.pop().unwrap();

        unsafe {
            if let Record(rhs) = right.unwrap() {
                let rhs = rhs.to_raw();
                if let Record(proto) = left.unwrap() {
                    if rhs == vm.drec.as_ref().unwrap().to_raw() {
                        vm.stack.push(Int(1).wrap());
                    } else {
                        let proto = &*proto.into_raw();
                        vm.stack
                            .push(Int(proto.is_prototype_of(&*rhs) as i64).wrap());
                    }
                } else if get_prototype(vm, left) == rhs {
                    vm.stack.push(Int(1).wrap());
                } else {
                    vm.stack.push(Int(0).wrap());
                }
            } else {
                vm.error = ERROR_EXPECTED_RECORD_OF_EXPR;
                vm.ip = (vm.ip as i32 - 1) as u32;
                return;
            }
        }
    }

    // variables
    // creates a new environment whenever a function is called
    // the environment is initialized with a copy of the current environment's
    if EnvNew == vm.code[vm.ip as usize] {
        log_debug!("EnvNew, Ip: {} sum(3)", vm.ip);
        let nslots =
            u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        unsafe {
            let last_entry = vm.localenv.last().unwrap();

            if let Some(env) = (**last_entry).borrow_mut().as_mut() {
                env.reserve(nslots);
                debug_assert!(vm.stack.len() >= env.nargs as usize);
                // Insert the args (sent by the user) into the environment.

                log_debug!("  send ars: {}", env.nargs);

                for i in 0..env.nargs {
                    let val = vm.stack.pop().expect("the bytecode interpreter does not have all the arguments that the user requires.");

                    env.set(i, val);
                }
            }
        }
    }

    // variables
    // sets the value of current environment's slot to the top of the stack
    if SetLocal == vm.code[vm.ip as usize] {
        log_debug!("SetLocal, IP: {}", vm.ip);
        let nslots =
            u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;

        unsafe {
            let last_entry = vm.localenv.last().unwrap();

            if let Some(env) = (**last_entry).borrow_mut().as_mut() {
                env.set(nslots, vm.stack[vm.stack.len() - 1]);
            }
        }
    }

    // this is for recursive function
    if SetLocalFunctionDef == vm.code[vm.ip as usize] {
        log_debug!("SetLocalFunctionDef, IP: {}", vm.ip);
        let nslots =
            u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        unsafe {
            let last_entry = vm.localenv.last().unwrap();
            // let env: &mut Env = vm.localenv.as_mut().unwrap().as_mut();
            if let Some(env) = (**last_entry).borrow_mut().as_mut() {
                env.set(nslots, vm.stack[vm.stack.len() - 1]);
                if let Fn(fun) = vm.stack[vm.stack.len() - 1].unwrap() {
                    if let Some(bound) = (*(*fun.into_raw()).bound).borrow_mut().as_mut() {
                        bound.set(nslots, vm.stack[vm.stack.len() - 1]);
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }

    // pushes a copy of the value of current environment's slot
    if GetLocal == vm.code[vm.ip as usize] {
        let slot = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        log_debug!("GetLocal, IP: {} sum(3), slot {}", vm.ip, slot);
        unsafe {
            let last_entry = vm.localenv.last().unwrap();

            // let env: &mut Env = vm.localenv.as_mut().unwrap().as_mut();
            if let Some(env) = (**last_entry).borrow().as_ref() {
                let value = env.get(slot);
                log_debug!("  value: {:?}", &value);

                vm.stack.push(value);
            }
        }
    }

    if GetLocalUp == vm.code[vm.ip as usize] {
        log_debug!("GetLocalUp, IP: {}", vm.ip);
        let slot = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);

        let relascope =
            u16::from_be_bytes([vm.code[vm.ip as usize + 3], vm.code[vm.ip as usize + 4]]);
        vm.ip += 5;

        unsafe {
            //let env: &mut Env = vm.localenv.as_mut().unwrap().as_mut();

            let last_entry = vm.localenv.last().unwrap();

            if let Some(env) = (**last_entry).borrow().as_ref() {
                vm.stack.push(env.get_up(relascope, slot));
            }
        }
    }

    // sets the value of the global variable to the top of the stack
    if SetGlobal == vm.code[vm.ip as usize] {
        log_debug!("SetGlobal, IP: {}", vm.ip);
        let key = generate_string(vm);
        log_debug!("  key: {}", &key);
        let obj = vm.stack[vm.stack.len() - 1];
        vm.mut_global().insert(key.into(), obj);
    }

    // pushes a copy of the value of the global variable
    // WARNING: This condition may not act as expected?
    if GetGlobal == vm.code[vm.ip as usize] {
        log_debug!("GetGlobal, IP: {}", vm.ip);
        let key = generate_string(vm);
        log_debug!("  key: {}", &key);

        let v: Option<&NativeValue> = { vm.global().get(&key) };

        //#[allow(mutable_borrow_reservation_conflict)]
        if let Some(val) = v {
            vm.stack.push(*val);
        } else {
            vm.error = ERROR_UNDEFINED_GLOBAL_VAR;
            vm.ip = (vm.ip as i32 - (key.len() as i32 + 2)) as u32;
            log_debug!("  IP: {}", &vm.ip);
            return;
        }
    }

    // pushes a function with [name], that begins at the next instruction pointer
    // to the stack and jumps to the [end address]
    if DefFunctionPush == vm.code[vm.ip as usize] {
        log_debug!("DefFunctionPush, IP: {}", vm.ip);
        // [opcode][end address]
        let nargs = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        let pos = u16::from_be_bytes([vm.code[vm.ip as usize + 3], vm.code[vm.ip as usize + 4]]);

        vm.ip += 3;
        log_debug!("  args: {}, pos: {} sum(3)", nargs, pos);

        unsafe {
            let env = match vm.localenv.last() {
                Some(e) => Rc::clone(e),
                None => Rc::default(),
            };
            // if vm.localenv.is_none() {
            //     std::ptr::null()
            // } else {
            //     vm.localenv.unwrap().as_ref()
            // };

            vm.stack
                .push(Fn(vm.malloc(Function::new(vm.ip + 2, nargs, env))).wrap());
        }

        vm.ip += pos as u32;
    }

    // flow control
    // jmp [32-bit position] (jump to back)
    if Jmp == vm.code[vm.ip as usize] {
        log_debug!("OP_JMP, IP: {}", vm.ip);
        vm.ip += 1;
        let pos = i16::from_be_bytes([vm.code[vm.ip as usize], vm.code[vm.ip as usize + 1]]);
        vm.ip = (vm.ip as i32 + pos as i32) as u32;
    }

    // jmp [32-bit position]
    if JmpLong == vm.code[vm.ip as usize] {
        log_debug!("OP_JMP_LONG, IP: {}", vm.ip);
        vm.ip += 1;
        let pos = u32::from_be_bytes([
            vm.code[vm.ip as usize],
            vm.code[vm.ip as usize + 1],
            vm.code[vm.ip as usize + 2],
            vm.code[vm.ip as usize + 3],
        ]);
        vm.ip = pos;
    }

    // jmp if not true [32-bit position]
    if [JCond as u8, JCondNoPop as u8].contains(&vm.code[vm.ip as usize]) {
        log_debug!("OP_JCOND/OP_JCOND_NO_POP, IP: {}", vm.ip);

        let val = if JCond == vm.code[vm.ip as usize] {
            vm.stack.pop().unwrap()
        } else {
            *vm.stack.last().unwrap()
        };
        vm.ip += 1;
        let pos = i16::from_be_bytes([vm.code[vm.ip as usize], vm.code[vm.ip as usize + 1]]);
        if value_is_true(val) {
            vm.ip = (vm.ip as i32 + pos as i32) as u32;
        } else {
            vm.ip += 2;
        }
    }

    // jump (to back) if true [32-bit position]
    // Jump-Condition JMP
    if [JNcond as u8, JNcondNoPop as u8].contains(&vm.code[vm.ip as usize]) {
        log_debug!("OP_JNCOND/OP_JNCOND_NO_POP");
        let val = if JNcond == vm.code[vm.ip as usize] {
            vm.stack.pop().unwrap()
        } else {
            *vm.stack.last().unwrap()
        };
        vm.ip += 1;
        let pos = i16::from_be_bytes([vm.code[vm.ip as usize], vm.code[vm.ip as usize + 1]]);

        if !value_is_true(val) {
            vm.ip = (vm.ip as i32 + pos as i32) as u32;
        } else {
            vm.ip += 2;
        }
    }

    if Call == vm.code[vm.ip as usize] {
        log_debug!("Call, IP: {}", vm.ip);

        let val = unsafe { vm.stack.pop().unwrap().unwrap() };
        let nargs = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        log_debug!("  sum(3), val: {}, nargs: {}", val, nargs);

        debug_assert!(vm.stack.len() >= nargs as usize);
        match val {
            NativeFn(native) => {
                vm.native_call_depth += 1;
                unsafe {
                    native(vm, nargs);
                }
                vm.native_call_depth -= 1;
                let call_depth = if vm.exframe_fallthrough.is_some() {
                    vm.exframe_fallthrough
                        .as_ref()
                        .unwrap()
                        .unwind_native_call_depth
                } else {
                    vm.native_call_depth
                };

                if call_depth != vm.native_call_depth || vm.error != ERROR_NO_ERROR {
                    return;
                }
            }
            Record(ref reco) => {
                let pctor = unsafe { (*reco.to_raw()).get("new") };
                if pctor.is_none() {
                    vm.error = ERROR_RECORD_NO_CONSTRUCTOR;
                    vm.ip = (vm.ip as i32 - 3) as u32;
                    return;
                }
                let ctor = pctor.unwrap();
                match unsafe { ctor.unwrap() } {
                    NativeFn(native) => {
                        vm.native_call_depth += 1;
                        unsafe {
                            native(vm, nargs);
                        }
                        vm.native_call_depth -= 1;
                        let call_depth = if vm.exframe_fallthrough.is_some() {
                            vm.exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                        } else {
                            vm.native_call_depth
                        };

                        if call_depth != vm.native_call_depth || vm.error != ERROR_NO_ERROR {
                            return;
                        }

                        if vm.exframe_fallthrough.is_some() {
                            if vm
                                .exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                                == vm.native_call_depth
                            {
                            } else {
                                return;
                            }
                        }
                    }
                    Fn(ifn) => {
                        let ifn = ifn.to_raw();
                        unsafe {
                            if nargs + 1 != (*ifn).nargs {
                                vm.error = ERROR_MISMATCH_ARGUMENTS;
                                vm.error_expected = (*ifn).nargs as u32;
                                vm.ip = (vm.ip as i32 - 3) as u32;
                                return;
                            }
                            let mut new_val = record::Record::new();
                            // vm.malloc().into_raw();
                            new_val.insert("prototype", val.wrap());
                            vm.stack.push(Record(vm.malloc(new_val)).wrap());
                            vm.enter_env(&*ifn);
                            // vm.localenv();
                        }
                    }
                    _ => {
                        vm.error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
                        vm.ip = (vm.ip as i32 - 3) as u32;
                        return;
                    }
                }
            }
            Fn(hfn) => unsafe {
                let ifn = hfn.to_raw();
                if nargs != (*ifn).nargs {
                    vm.error = ERROR_MISMATCH_ARGUMENTS;
                    vm.ip = (vm.ip as i32 - 3) as u32;
                    vm.error_expected = (*ifn).nargs as u32;
                    return;
                }
                vm.enter_env(&*ifn);
                // vm.localenv();
            },
            _ => {
                vm.error = ERROR_EXPECTED_CALLABLE;
                vm.ip = (vm.ip as i32 - 3) as u32;
                return;
            }
        }
    }

    // returns from function
    if Ret == vm.code[vm.ip as usize] {
        log_debug!("Ret, IP: {}", vm.ip);
        let last_entry = Rc::clone(vm.localenv.last().unwrap());

        if let Some(env) = (*last_entry).borrow().as_ref() {
            if env.retip == u32::MAX {
                //LOG("return from vm_call\n");
                return;
            }
        }
        vm.leave_env();

        //LOG("ip = %d\n", vm->ip);
    }

    // dictionaries
    if DictNew == vm.code[vm.ip as usize] {
        log_debug!("OP_DICT_NEW");
        vm.ip += 1;
        unimplemented!();
    }

    if MemberGet == vm.code[vm.ip as usize] || MemberGetNoPop == vm.code[vm.ip as usize] {
        log_debug!("MemberGet/MemberGetNoPop");
        let op = vm.code[vm.ip as usize];

        //vm->ip += (uint32_t)strlen(key) + 1;
        let val = unsafe { vm.stack.last().unwrap().unwrap() };

        let pos = vm.ip;
        let key = generate_string(vm);

        unsafe {
            let dict: *const record::Record;
            if let Record(reco) = val {
                dict = reco.to_raw();
                if op == MemberGet as u8 {
                    vm.stack.pop();
                }
            } else {
                dict = get_prototype(vm, val.wrap());
                if dict.is_null() {
                    vm.error = ERROR_CANNOT_ACCESS_NON_RECORD;
                    vm.ip = (pos as i32 - 1) as u32;
                    return;
                }
                if key == *"prototype" {
                    if op == MemberGet as u8 {
                        vm.stack.pop();
                    }

                    vm.stack
                        .push(Record(Gc::from_raw(dict as *mut record::Record)).wrap());
                    return; // Does this go here?
                }
            }

            let result = (*dict).get(&key);
            if let Some(result) = result {
                vm.stack.push(*result);
            } else {
                vm.error = ERROR_UNKNOWN_KEY;
                vm.ip = (pos as i32 - 1) as u32; // or? vm.ip = pos;
                return;
            }
        }
    }

    if MemberSet == vm.code[vm.ip as usize] {
        log_debug!("MemberSet, IP: {}", vm.ip);
        // stack: [value][dict]
        let pos = vm.ip;
        let key = generate_string(vm);

        let dval = unsafe { vm.stack.last().copied().unwrap().unwrap() };
        match dval {
            Record(mut reco) => {
                vm.stack.pop();
                let val = vm.stack.pop().unwrap();
                reco.inner_mut_ptr().insert(key, val);
            }
            _ => {
                vm.error = ERROR_CANNOT_ACCESS_NON_RECORD;
                vm.ip = pos;
                return;
            }
        }
    }

    if DictLoad == vm.code[vm.ip as usize] {
        log_debug!("DictLoad, IP: {}", vm.ip);
        // stack: [nil][value][key]
        vm.ip += 1;

        let mut length = unsafe {
            let val = vm.stack.pop().unwrap().unwrap();
            if let Int(num) = val {
                num as usize
            } else {
                unreachable!("Expect integer, found {}", val.type_name());
            }
        };

        let mut dval = record::Record::with_capacity(length);

        while length > 0 {
            //debug_assert(key.type == TYPE_STR);
            // key
            let key = unsafe {
                let val = vm.stack.pop().unwrap().unwrap();
                if let Str(s) = val {
                    s
                } else {
                    unreachable!("Expect string, found {}", val.type_name());
                }
            };
            // val
            let val = vm.stack.pop().unwrap();
            let key = unsafe { (*key.to_raw()).borrow() } as &String;
            dval.insert(key.clone(), val);

            length -= 1;
        }

        vm.stack.push(Record(vm.malloc(dval)).wrap());
    }

    if ArrayLoad == vm.code[vm.ip as usize] {
        log_debug!("ArrayLoad, IP: {}", vm.ip);
        vm.ip += 1;

        let mut length = unsafe {
            let val = vm.stack.pop().unwrap().unwrap();
            if let Int(num) = val {
                num as usize
            } else {
                unreachable!("Expect integer, found {}", val.type_name());
            }
        };

        if length == 0 {
            vm.stack.push(Array(vm.malloc(Vec::new())).wrap());
        } else {
            let mut array = Vec::with_capacity(length);

            while length > 0 {
                array.insert(0, vm.stack.pop().unwrap());
                length -= 1
            }
            vm.stack.push(Array(vm.malloc(array)).wrap());
        }
    }

    // exceptions
    if Try == vm.code[vm.ip as usize] {
        log_debug!("Try, IP: {}", vm.ip);
        // stack: [nil][function][error type]
        //LOG("TRY\n");
        vm.ip += 1;

        let frame: *mut _ = vm.enter_exframe();
        let mut error;
        loop {
            unsafe {
                error = vm.stack.last().unwrap().unwrap();
                if Nil == error {
                    break;
                }
                // error type
                if let Record(reco) = error {
                    vm.stack.pop();
                    // val
                    let xfn = vm.stack.last().unwrap().unwrap();
                    let xfn = if let Fn(f) = xfn {
                        f.to_raw()
                    } else {
                        unreachable!();
                    };
                    //debug_assert!(xfn.type == TYPE_FN);
                    vm.stack.pop();
                    (*frame).set_handler(reco.inner_ptr(), (*xfn).clone());
                } else {
                    vm.error = ERROR_CASE_EXPECTS_DICT;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    return;
                }
            }
        }
        vm.stack.pop(); // pop nil
    }

    if Raise == vm.code[vm.ip as usize] {
        log_debug!("Raise, IP: {}", vm.ip);
        if !vm.raise() {
            vm.error = ERROR_UNHANDLED_EXCEPTION;
            if vm.exframe_fallthrough.is_some() || vm.native_call_depth != 0 {
                log_debug!("falling through pls wait ({})\n", vm.native_call_depth);
                return;
            }
            return;
        }

        if vm.exframe_fallthrough.is_some() || vm.native_call_depth != 0 {
            log_debug!("falling through pls wait ({})\n", vm.native_call_depth);
            return;
        }
    }

    if ExframeRet == vm.code[vm.ip as usize] {
        log_debug!("ExframeRet, IP: {}", vm.ip);
        vm.ip += 1;
        let pos = u16::from_be_bytes([vm.code[vm.ip as usize], vm.code[vm.ip as usize + 1]]);
        vm.ip += pos as u32;
        vm.leave_exframe();
    }

    if RetCall == vm.code[vm.ip as usize] {
        log_debug!("RetCall, IP: {}", vm.ip);
        let val = unsafe { vm.stack.last().unwrap().unwrap() };
        let nargs = u16::from_be_bytes([vm.code[vm.ip as usize + 1], vm.code[vm.ip as usize + 2]]);
        vm.ip += 3;
        debug_assert!(vm.stack.len() >= (nargs as usize));

        match val {
            NativeFn(native) => {
                vm.stack.pop();
                vm.native_call_depth += 1;
                unsafe {
                    native(vm, nargs);
                }
                vm.native_call_depth -= 1;
                let call_depth = if vm.exframe_fallthrough.is_some() {
                    vm.exframe_fallthrough
                        .as_ref()
                        .unwrap()
                        .unwind_native_call_depth
                } else {
                    vm.native_call_depth
                };

                if call_depth != vm.native_call_depth || vm.error != ERROR_NO_ERROR {
                    return;
                }

                let last_entry = Rc::clone(vm.localenv.last().unwrap());

                if let Some(env) = (*last_entry).borrow().as_ref() {
                    if env.retip == u32::MAX {
                        return;
                    }
                }
                vm.leave_env();
            }
            Fn(hfn) => unsafe {
                vm.stack.pop();
                let ifn = hfn.to_raw();
                if nargs != (*ifn).nargs {
                    vm.error = ERROR_MISMATCH_ARGUMENTS;
                    vm.ip = (vm.ip as i32 - 3) as u32;
                    vm.error_expected = (*ifn).nargs as u32;
                    return;
                }

                vm.enter_env_tail(&*ifn);
                // vm.localenv();
            },
            Record(ref reco) => {
                let pctor = unsafe { (*reco.to_raw()).get("constructor") };
                if pctor.is_none() {
                    vm.error = ERROR_RECORD_NO_CONSTRUCTOR;
                    vm.ip = (vm.ip as i32 - 3) as u32;
                    return;
                }
                let ctor = pctor.unwrap();
                match unsafe { ctor.unwrap() } {
                    NativeFn(native) => {
                        vm.native_call_depth += 1;
                        unsafe {
                            native(vm, nargs);
                        }
                        vm.native_call_depth -= 1;
                        let call_depth = if vm.exframe_fallthrough.is_some() {
                            vm.exframe_fallthrough
                                .as_ref()
                                .unwrap()
                                .unwind_native_call_depth
                        } else {
                            vm.native_call_depth
                        };

                        if call_depth != vm.native_call_depth || vm.error != ERROR_NO_ERROR {
                            return;
                        }

                        let last_entry = vm.localenv.last().unwrap();

                        if let Some(env) = (**last_entry).borrow().as_ref() {
                            if env.retip == u32::MAX {
                                //LOG("return from vm_call\n");
                                return;
                            }
                        }
                    }
                    Fn(ifn) => {
                        let ifn = ifn.to_raw();
                        unsafe {
                            if nargs + 1 != (*ifn).nargs {
                                vm.error = ERROR_MISMATCH_ARGUMENTS;
                                vm.error_expected = (*ifn).nargs as u32;
                                vm.ip = (vm.ip as i32 - 3) as u32;
                                return;
                            }
                            let mut new_val = record::Record::new();
                            new_val.insert("prototype", val.wrap());
                            vm.stack.push(Record(vm.malloc(new_val)).wrap());
                            vm.enter_env_tail(&*ifn);
                            // vm.localenv();
                        }
                    }
                    _ => {
                        vm.error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
                        vm.ip = (vm.ip as i32 - 3) as u32;
                        return;
                    }
                }
            }
            _ => {
                vm.error = ERROR_EXPECTED_CALLABLE;
                vm.ip = (vm.ip as i32 - 3) as u32;
                return;
            }
        }
    }

    // Remember the -2
    if ForIn == vm.code[vm.ip as usize] {
        log_debug!("ForIn, IP: {}", vm.ip);

        vm.ip += 1;
        let pos = u16::from_be_bytes([vm.code[vm.ip as usize], vm.code[vm.ip as usize + 1]]);
        // as usize + 1]);
        unsafe {
            //println!("{:?}, {}", vm.code, vm.ip);
            debug_assert!(!vm.stack.is_empty());
            let top = vm.stack.last().unwrap().unwrap();
            match top {
                Str(xstr) => {
                    let mut vec = (*xstr.to_raw())
                        .deref()
                        .chars()
                        .map(|ch| Str(vm.malloc(ch.to_string().into())).wrap())
                        .collect::<Vec<_>>();
                    vm.stack.pop();
                    if vec.is_empty() {
                        // skip empty
                        vm.ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                    } else {
                        let less = vec.remove(0);
                        vm.stack.push(Array(vm.malloc(vec)).wrap());
                        vm.stack.push(Iterator.wrap());
                        vm.stack.push(less);
                    }
                }
                Array(array) => {
                    let array = array.to_raw();
                    if (*array).is_empty() {
                        // skip empty
                        vm.ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                        vm.stack.pop();
                    } else {
                        vm.stack.pop();
                        let mut clone = (*array).clone();
                        let less = clone.remove(0);

                        vm.stack.push(Array(vm.malloc(clone)).wrap()); // De esta manera no consumiremos el original
                        vm.stack.push(Iterator.wrap());
                        vm.stack.push(less);
                    }
                }
                //TYPE_DICT
                // interation
                Iterator => {
                    // There must be at least two values on the stack!
                    debug_assert!(vm.stack.len() >= 2);

                    let iterator = vm.stack[vm.stack.len() - 2];
                    match iterator.unwrap() {
                        Nil => {
                            vm.ip += (pos as i32 - 2) as u32;
                        }

                        // NativeValue
                        Array(arr) => {
                            let arr = arr.into_raw();
                            if (*arr).is_empty() {
                                vm.stack.pop(); /* iterator */
                                vm.stack.pop(); /* array */
                                vm.ip += (pos as i32 - 2) as u32; // -2 sizeof(pos)
                            } else {
                                log_debug!("CONTINUE\n");
                                //vm.stack.pop(); /* old iterator */
                                //array_push(vm->stack, value_pointer(TYPE_INTERPRETER_ITERATOR,
                                // (void *)(idx + 1)));
                                // vm.stack.push(Iterator.wrap());
                                vm.stack.push((*arr).remove(0));
                            }
                        }

                        _ => {
                            vm.error = ERROR_EXPECTED_ITERABLE;
                            // 1 + sizeof(pos) (where 1 is the operator)
                            vm.ip = (vm.ip as i32 - 3) as u32; // R(siz)()?
                            return;
                        }
                    }
                }
                _ => {
                    log_debug!("NOT ITERABLE\n");
                    vm.error = ERROR_EXPECTED_ITERABLE;
                    // -2 == - sizeof(pos)
                    vm.ip = (vm.ip as i32 - 2) as u32; // R(siz)()?
                    return;
                }
            }
            vm.ip += 2; // Sera un descuido R(siz)()?
        }
    }

    if IndexGet == vm.code[vm.ip as usize] || MemberGetNoPop == vm.code[vm.ip as usize] {
        log_debug!("IndexGET/MemberGetNoPop, IP: {}", vm.ip);
        let index = unsafe { vm.stack.pop().unwrap().unwrap() };
        let dval = unsafe {
            if IndexGet == vm.code[vm.ip as usize] {
                vm.stack.pop().unwrap()
            } else {
                vm.stack.last().copied().unwrap()
            }
            .unwrap()
        };

        vm.ip += 1;
        match dval {
            Array(array) => {
                // Note: i64::MAX < usize::MAX
                let array = unsafe { &*array.to_raw() };
                let index = if let Int(num) = index {
                    if num < 0 {
                        ((array.len() as i64) + num) as usize
                    } else {
                        num as usize
                    }
                } else {
                    vm.error = ERROR_KEY_NON_INT;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    return;
                };

                if index >= array.len() {
                    vm.error = ERROR_UNBOUNDED_ACCESS;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    vm.error_expected = (array.len()) as u32;
                    return;
                }

                vm.stack.push(array[index])
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
                    vm.error = ERROR_KEY_NON_INT;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    return;
                };

                let left = if let Some(ch) = xstr.graphemes(true).nth(index) {
                    vm.malloc(ch.to_string().into())
                } else {
                    vm.error = ERROR_UNBOUNDED_ACCESS;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    vm.error_expected = len as u32;
                    return;
                };
                vm.stack.push(Str(left).wrap());
            }
            _ => {
                vm.error = ERROR_CANNOT_ACCESS_NON_RECORD;
                vm.ip = (vm.ip as i32 - 1) as u32;
                return;
            }
        }
    }

    if IndexSet == vm.code[vm.ip as usize] {
        log_debug!("IndexSet, IP: {}", vm.ip);
        vm.ip += 1;

        let index = unsafe { vm.stack.pop().unwrap().unwrap() };
        let dval = unsafe { vm.stack.pop().unwrap().unwrap() };
        let val = vm.stack.last().unwrap();

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
                    vm.error = ERROR_KEY_NON_INT;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    return;
                };

                if index >= array.len() {
                    vm.error = ERROR_UNBOUNDED_ACCESS;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    vm.error_expected = (array.len()) as u32;
                    return;
                }

                array[index] = *val;
            }
            // TODO: the Record's should be more like classes
            // than dictionaries, in the future, Hana will support dictionaries.
            // dictionaries, at the moment we will continue, with the old
            // traditions :(.
            Record(mut reco) => unsafe {
                let index = if let Str(s) = index {
                    (*s.into_raw()).clone()
                } else {
                    vm.error = ERROR_RECORD_KEY_NON_STRING;
                    vm.ip = (vm.ip as i32 - 1) as u32;
                    return;
                };

                reco.inner_mut_ptr().insert(index, *val);
            },
            _ => {
                vm.error = ERROR_EXPECTED_RECORD_ARRAY;
                vm.ip = (vm.ip as i32 - 1) as u32;
                return;
            }
        }
    }

    if Swap == vm.code[vm.ip as usize] {
        log_debug!("Swap, IP: {}", vm.ip);
        debug_assert!(vm.stack.len() >= 2);
        vm.ip += 1;
        let one = vm.stack.len() - 1;
        let two = vm.stack.len() - 2;
        let lower = vm.stack[two];
        let higher = vm.stack[one];
        vm.stack[one] = lower;
        vm.stack[two] = higher;
    }

    // modules
    if Use == vm.code[vm.ip as usize] {
        log_debug!("Use, IP: {}", vm.ip);
        let path = generate_string(vm);
        vm.load_module(&path);
    }

    inside_execute(vm);
}

pub(super) fn vm_call(vm: &mut Vm, func: NativeValue, args: &[NativeValue]) -> NativeValue {
    let ifn: *mut Function = std::ptr::null_mut();
    let nargs = args.len() as u16;
    let func = unsafe { func.unwrap() };
    // (fn.type == TYPE_NATIVE_FN)

    if let NativeFn(value_fn) = func {
        let len = args.len() as isize - 1;
        for i in -len..1 {
            vm.stack.push(args[(-i) as usize]);
        }
        unsafe {
            value_fn(vm, nargs);
        }
        return vm.stack.pop().unwrap();
    } else if let Record(pctor) = func.clone() {
        let pctor = unsafe { (*pctor.to_raw()).get("constructor") };

        if pctor.is_none() {
            vm.error = ERROR_RECORD_NO_CONSTRUCTOR;
            return NativeValue {
                r#type: TYPE_INTERPRETER_ERROR,
                data: 0,
            };
        }

        if let NativeFn(value_fn) = unsafe { pctor.unwrap().unwrap() } {
            let len = args.len() as isize - 1;
            for i in -len..1 {
                vm.stack.push(args[(-i) as usize]);
            }
            unsafe {
                value_fn(vm, nargs);
            }

            return vm.stack.pop().unwrap();
        } else if let Fn(f) = func {
            unsafe { *ifn = (*f.into_raw()).clone() };
        } else {
            vm.error = ERROR_CONSTRUCTOR_NOT_FUNCTION;
            return NativeValue {
                r#type: TYPE_INTERPRETER_ERROR,
                data: 0,
            };
        }
    } else if let Fn(f) = func {
        unsafe { *ifn = (*f.into_raw()).clone() };
    } else {
        vm.error = ERROR_EXPECTED_CALLABLE;
        return NativeValue {
            r#type: TYPE_INTERPRETER_ERROR,
            data: 0,
        };
    }
    unsafe {
        if nargs != (*ifn).nargs {
            vm.ip = (*ifn).ip;
            vm.error = ERROR_MISMATCH_ARGUMENTS;
            return NativeValue {
                r#type: TYPE_INTERPRETER_ERROR,
                data: 0,
            };
        }
    }

    let last = vm.ip;
    // setup env
    let oldenv = vm.localenv.len();
    vm.ip = (vm.ip as i32 - 1) as u32;
    unsafe {
        vm.enter_env(&*ifn);
    }
    let curenv = vm.localenv.len();
    // setup stack/ip

    let len = args.len() as isize - 1;
    for i in -len..1 {
        vm.stack.push(args[(-i) as usize]);
    }

    inside_execute(vm);
    if vm.error != ERROR_NO_ERROR || vm.exframe_fallthrough.is_some() {
        // exception
        //LOG("falling through psl wait %ld", vm->native_call_depth);
        return NativeValue {
            r#type: TYPE_INTERPRETER_ERROR,
            data: 0,
        };
    }

    if vm.localenv.len() != curenv {
        // exception occurred outside of function's scope
        // NOTE: curenv already free'd from unwinding
        return NativeValue {
            r#type: TYPE_INTERPRETER_ERROR,
            data: 0,
        };
    }

    vm.localenv = vm.localenv.drain(..oldenv).collect();
    vm.ip = last;

    vm.stack.pop().unwrap()
}
