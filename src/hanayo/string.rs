//! Provides String record for handling UTF-8 strings
extern crate unicode_segmentation;
use crate::harumachine::value::Value;
use crate::harumachine::vm::Vm;
use crate::harumachine::vmerror::VmError;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;

/// # Safety
///
/// This method must be unsafe for interoperability with other languages.
pub fn constructor(vm: Rc<RefCell<Vm>>, nargs: u16) {
    
    if nargs == 0 {
        vm.borrow_mut().stack
            .push(Value::Str((*vm).borrow().malloc(String::new().into())));
    } else if nargs == 1 {
        let arg = vm.borrow_mut().stack.pop().unwrap();
        vm.borrow_mut().stack
            .push(Value::Str((*vm).borrow().malloc(format!("{}", arg).to_string().into())));
    } else {
        vm.borrow_mut().error = VmError::ERROR_MISMATCH_ARGUMENTS;
        vm.borrow_mut().error_expected = 1;
    }
}

// length
#[hana_function]
fn length(s: Value::Str) -> Value {
    Value::Int(s.as_ref().graphemes(true).count() as i64)
}
#[hana_function]
fn bytesize(s: Value::Str) -> Value {
    Value::Int(s.as_ref().len() as i64)
}

// check
#[hana_function]
fn startswith(s: Value::Str, left: Value::Str) -> Value {
    let s = s.as_ref().borrow() as &String;
    Value::Int(s.starts_with(left.as_ref().borrow() as &String) as i64)
}
#[hana_function]
fn endswith(s: Value::Str, left: Value::Str) -> Value {
    let s = s.as_ref().borrow() as &String;
    Value::Int(s.ends_with(left.as_ref().borrow() as &String) as i64)
}

// basic manip
#[hana_function]
fn delete(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let ss = s
        .as_ref()
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, ch)| {
            if nchars == -1 {
                if i >= from_pos {
                    None
                } else {
                    Some(ch)
                }
            } else if (from_pos..(from_pos + nchars as usize)).contains(&i) {
                None
            } else {
                Some(ch)
            }
        })
        .collect::<String>();

    Value::Str((*vm).borrow().malloc(ss.into()))
}

#[hana_function]
fn delete_(mut s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let it = s.as_ref().grapheme_indices(true).skip(from_pos);
    if let Some((i, _)) = it.clone().take(1).next() {
        if nchars == -1 {
            s.inner_mut_ptr().truncate(i);
        } else if let Some((j, _)) = it.skip(nchars as usize).take(1).next() {
            s.inner_mut_ptr().replace_range(i..j, "");
        } else {
            s.inner_mut_ptr().remove(i);
        }
    }
    Value::Str(s)
}

#[hana_function]
fn copy(s: Value::Str, from_pos: Value::Int, nchars: Value::Int) -> Value {
    let from_pos = from_pos as usize;
    let ss = s
        .as_ref()
        .graphemes(true)
        .enumerate()
        .filter_map(|(i, ch)| {
            if nchars == -1 {
                if i >= from_pos {
                    Some(ch)
                } else {
                    None
                }
            } else if (from_pos..(from_pos + nchars as usize)).contains(&i) {
                Some(ch)
            } else {
                None
            }
        })
        .collect::<String>();

    Value::Str((*vm).borrow().malloc(ss.into()))
}

#[hana_function]
fn insert_(mut dst: Value::Str, from_pos: Value::Int, src: Value::Str) -> Value {
    if let Some((i, _)) = dst.as_ref().grapheme_indices(true).nth(from_pos as usize) {
        dst.inner_mut_ptr().insert_str(i, src.as_ref().as_str());
    }

    Value::Str(dst)
}

// other
#[hana_function]
fn split(s: Value::Str, delim: Value::Str) -> Value {
    let mut array = (*vm).borrow().malloc(Vec::new());
    let s = s.as_ref().borrow() as &String;
    for ss in s.split(delim.as_ref().borrow() as &String) {
        array
            .inner_mut_ptr()
            .push(Value::Str((*vm).borrow().malloc(ss.to_string().into())));
    }

    Value::Array(array)
}

#[hana_function]
fn index(s: Value::Str, needle: Value::Str) -> Value {
    let s: &String = s.as_ref().borrow();
    match s.find(needle.as_ref().borrow() as &String) {
        Some(x) => Value::Int({
            let mut idx_grapheme = 0;
            if s.grapheme_indices(true).any(|(i, _)| {
                idx_grapheme += 1;
                i == x
            }) {
                (idx_grapheme - 1) as i64
            } else {
                -1
            }
        }),
        None => Value::Int(-1),
    }
}

#[hana_function]
fn chars(s: Value::Str) -> Value {
    let mut array = (*vm).borrow().malloc(Vec::new());
    let array_ref = array.inner_mut_ptr();
    
    for ch in s.as_ref().graphemes(true) {
        array_ref.push(Value::Str((*vm).borrow().malloc(ch.to_string().into())));
    }

    Value::Array(array)
}

#[hana_function]
fn ord(s: Value::Str) -> Value {
    let s = s.as_ref();
    if let Some(ch) = s.chars().next() {
        Value::Int(ch as i64)
    } else {
        Value::Int(0)
    }
}
