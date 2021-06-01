//! Bindings for the virtual machine.
#![allow(clippy::from_over_into, clippy::float_cmp, clippy::new_without_default)]
#![allow(clippy::fn_to_numeric_cast, clippy::missing_safety_doc)]

pub mod env;
pub mod exframe;
pub mod function;
pub mod gc;
pub mod hmap;
mod inside;
pub mod interned_string_map;
pub mod nativeval;
pub mod operations;
pub mod record;
pub mod string;
pub mod value;
pub mod vm;
pub mod vmerror;
