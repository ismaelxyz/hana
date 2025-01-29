//! Provides Array record for handling arrays
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use crate::harumachine::nativeval::NativeValue;
use crate::harumachine::value::Value;
use crate::harumachine::{
    operations::{value_eq, value_gt, value_lt},
    vm::Vm,
};

/// # Safety
///
/// This function needs to be unsafe for internal compatibility between multiple languages.
pub fn constructor(vm: Rc<RefCell<Vm>>, nargs: u16) {
    if nargs == 0 {
        let new_array = Value::Array(vm.borrow().malloc(Vec::new()));
        vm.borrow_mut().stack.push(new_array).wrap();
        return;
    }

    let nargs = nargs as usize;
    let mut array = vm.malloc(Vec::with_capacity(nargs));
    for _i in 0..nargs {
        let val = vm.stack.pop().unwrap();
        array.inner_mut_ptr().push(val);
    }
    vm.stack.push(Value::Array(array).wrap());
}

#[hana_function()]
fn length(array: Value::Array) -> Value {
    Value::Int(array.as_ref().len() as i64)
}

#[hana_function()]
fn insert_(mut array: Value::Array, pos: Value::Int, elem: Value::Any) -> Value {
    array.inner_mut_ptr().insert(pos as usize, elem.wrap());
    Value::Int(array.as_ref().len() as i64)
}

#[hana_function()]
fn delete_(mut array: Value::Array, from_pos: Value::Int, nelems: Value::Int) -> Value {
    array
        .inner_mut_ptr()
        .drain((from_pos as usize)..((nelems as usize) + 1));
    Value::Int(array.as_ref().len() as i64)
}

// stack manipulation
#[hana_function()]
fn push(mut array: Value::Array, elem: Value::Any) -> Value {
    array.inner_mut_ptr().push(elem.wrap());
    Value::Nil
}

#[hana_function()]
fn pop(mut array: Value::Array) -> Value {
    unsafe { array.inner_mut_ptr().pop().unwrap().unwrap() }
}

// sorting
fn value_cmp(left: &NativeValue, right: &NativeValue) -> Ordering {
    let (left, right) = (*left, *right);

    match unsafe { value_gt(left.unwrap(), right.unwrap()).unwrap() } {
        Value::Int(1) => Ordering::Greater,
        _ => match unsafe { value_lt(left.unwrap(), right.unwrap()).unwrap() } {
            Value::Int(1) => Ordering::Less,
            _ => Ordering::Equal,
        },
    }
}

#[hana_function()]
fn sort(array: Value::Array) -> Value {
    let mut new_array = vm.malloc(array.as_ref().clone());
    let slice = new_array.inner_mut_ptr().as_mut_slice();
    slice.sort_by(value_cmp);
    Value::Array(new_array)
}
#[hana_function()]
fn sort_(mut array: Value::Array) -> Value {
    let slice = array.inner_mut_ptr().as_mut_slice();
    slice.sort_by(value_cmp);
    Value::Array(array)
}

// functional
#[hana_function()]
fn map(array: Value::Array, fun: Value::Any) -> Value {
    let mut new_array = vm.malloc(Vec::with_capacity(array.as_ref().len()));
    let mut args = Vec::with_capacity(1);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(*val);
        if let Some(val) = vm.call(fun.wrap(), &args) {
            new_array.inner_mut_ptr().push(val);
        } else {
            return Value::PropagateError;
        }
    }
    Value::Array(new_array)
}

#[hana_function()]
fn filter(array: Value::Array, fun: Value::Any) -> Value {
    let mut new_array = vm.malloc(Vec::new());
    let mut args = Vec::with_capacity(1);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(*val);
        if let Some(filter) = vm.call(fun.wrap(), &args) {
            if unsafe { filter.unwrap() }.is_true() {
                new_array.inner_mut_ptr().push(*val);
            }
        } else {
            return Value::PropagateError;
        }
    }
    Value::Array(new_array)
}

#[hana_function()]
fn reduce(array: Value::Array, fun: Value::Any, acc_: Value::Any) -> Value {
    let mut acc = acc_.clone();
    let mut args = Vec::with_capacity(2);
    for val in array.as_ref().iter() {
        args.clear();
        args.push(acc.wrap());
        args.push(*val);
        if let Some(val) = vm.call(fun.wrap(), &args) {
            acc = unsafe { val.unwrap() };
        } else {
            return Value::PropagateError;
        }
    }
    acc
}

// search
#[hana_function()]
fn index(array: Value::Array, elem: Value::Any) -> Value {
    let array = array.as_ref();
    // NOTE: array.len() -1
    for (i, item) in array.iter().enumerate() {
        if let Value::Int(1) = unsafe { value_eq(item.unwrap(), elem.clone()).unwrap() } {
            return Value::Int(i as i64);
        }
    }
    Value::Int(-1)
}

// strings
#[hana_function()]
fn join(array: Value::Array, delim: Value::Str) -> Value {
    let mut s = String::new();
    let array = array.as_ref();
    if !array.is_empty() {
        s += unsafe { format!("{}", array[0].unwrap()).as_str() };
    }
    if array.len() > 1 {
        let mut i = 1;
        while i < array.len() {
            s += delim.as_ref();
            s += unsafe { format!("{}", array[i].unwrap()).as_str() };
            i += 1;
        }
    }
    Value::Str(vm.malloc(s.into()))
}
