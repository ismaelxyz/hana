//! Interpreter for the hana programming language

//#![allow(clippy::not_unsafe_ptr_arg_deref)]
//#![feature(alloc_layout_extra)]
//#![feature(core_intrinsics)]
// #![feature(print_internals)]
// #![feature(format_args_nl)]

#[macro_use]
extern crate decorator;

#[macro_use]
extern crate cfg_if;

pub mod ast;
pub mod compiler;
pub mod grammar;
pub mod hanayo;
pub mod vmbindings;
