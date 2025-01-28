use crate::harumachine::nativeval::{NativeValue, NativeValueType::*};
use crate::harumachine::value::{Value, Value::*};
use crate::harumachine::{string::HaruString, vm::Vm};
use std::borrow::Borrow;
// Logical and mathematical operations on the values.

// All these operations will be eliminated and I will create a system where the
// type itself will be
// type is added, subtracted ... with the appropriate type instead of primitive
// functions to take care of it.
// to take care of it
pub(crate) fn value_add(left: Value, right: Value, vm: &mut Vm) -> NativeValue {
    match (&left, &right) {
        (Str(s), Str(s1)) => {
            let st: HaruString = unsafe {
                format!(
                    "{}{}",
                    s.to_raw().as_ref().unwrap().borrow() as &String,
                    s1.to_raw().as_ref().unwrap().borrow() as &String
                )
            }
            .into();
            let key = vm.malloc(st);
            Str(key).wrap()
        }
        (Int(i), Int(i2)) => Int(i + i2).wrap(),
        (Float(f), Float(f2)) => Float(f + f2).wrap(),
        (Int(i), Float(f)) | (Float(f), Int(i)) => Float(*i as f64 + f).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
// Original
pub(crate) fn value_sub(left: Value, right: Value, _: &mut Vm) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i2)) => Int(i - i2).wrap(),
        (Float(f), Float(f2)) => Float(f - f2).wrap(),
        (Int(i), Float(f)) => Float(*i as f64 - f).wrap(),
        (Float(f), Int(i)) => Float(f - *i as f64).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_mul(left: Value, right: Value, _: &mut Vm) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i2)) => Int(i * i2).wrap(),
        (Float(f), Float(f2)) => Float(f * f2).wrap(),
        (Int(i), Float(f)) => Float(*i as f64 * f).wrap(),
        (Float(f), Int(i)) => Float(f * (*i as f64)).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
/*
pub(crate) fn value_pow(left: Value, right: Value, vm: &mut Vm) -> NativeValue {


    match (&left, &right) {
        (Int(i), Int(i2)) => Int(i.pow(*i2 as u32)),
        (Float(f), Float(f2)) => Float(f.powf(*f2)),
        (Int(i), Float(f)) => Float(i.pow(*i as f64)),
        (Float(f), Int(i)) => Float(f.powf(*f as u32) as f64),
        _ => NativeValue {data: 0, r#type: TYPE_INTERPRETER_ERROR},
    }
}
*/

pub(crate) fn value_div(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        /*
        (Int(i), _) if i == &0 => {
            vm.error = ERROR_ZERO_DIVISION;
            vm.ip -= 1;
            vm.ip =
                (vm.ip as usize - (forward(&right) + forward(&left))) as u32;
            vm.stack.push(right);
            vm.stack.push(left);
            //vm.stack.push(InterpreterError);
        }
        (Float(f), _) if f == &0.0 => {
            vm.error = ERROR_ZERO_DIVISION;
            vm.ip -= 1;
            vm.ip =
                (vm.ip as usize - (forward(&right) + forward(&left))) as u32;
            vm.stack.push(right);
            vm.stack.push(left);
            //vm.stack.push(InterpreterError);
        }
        */
        (Int(i), Int(i2)) => Int(i / i2).wrap(),
        (Float(f), Float(f2)) => Float(f / f2).wrap(),
        (Int(i), Float(f)) => Float(*i as f64 / f).wrap(),
        (Float(f), Int(i)) => Float(f / *i as f64).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_mod(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i2)) => Int(i % i2).wrap(),
        (Float(f), Float(f2)) => Float(f % f2).wrap(),
        (Int(i), Float(f)) => Float(*i as f64 % f).wrap(),
        (Float(f), Int(i)) => Float(f % *i as f64).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_bitwise_and(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i2)) => Int(i & i2).wrap(),
        //(Bool(b), Bool(b1)) => Bool(b & b1).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
pub(crate) fn value_bitwise_or(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int(i | i1).wrap(),
        //(Bool(b), Bool(b1)) => Bool(b | b1).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_bitwise_xor(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int(i ^ i1).wrap(),
        //(Bool(b), Bool(b1)) => Bool(b ^ b1).wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_lt(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i < i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f < f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) < *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f < (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
pub(crate) fn value_leq(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i <= i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f <= f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) <= *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f <= (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
pub(crate) fn value_gt(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i > i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f > f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) > *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f > (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
pub(crate) fn value_geq(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i >= i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f >= f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) >= *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f >= (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
pub(crate) fn value_eq(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i == i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f == f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) == *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f == (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}

pub(crate) fn value_neq(left: Value, right: Value) -> NativeValue {
    match (&left, &right) {
        (Int(i), Int(i1)) => Int((i != i1) as i64).wrap(),
        (Float(f), Float(f1)) => Int((f != f1) as i64).wrap(),
        (Int(i), Float(f)) => Int(((*i as f64) != *f) as i64).wrap(),
        (Float(f), Int(i)) => Int((*f != (*i as f64)) as i64).wrap(),
        //(Str(s), Str(s)) =>Int().wrap(),
        _ => NativeValue {
            data: 0,
            r#type: TYPE_INTERPRETER_ERROR,
        },
    }
}
