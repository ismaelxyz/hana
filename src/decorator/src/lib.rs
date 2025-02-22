//! Decorator module for creating native functions callable from haru
//!
//! ```rust,text
//! extern crate haru;
//! use haru::vmbindings::vm::Vm;
//! use haru::vmbindings::value::Value;
//!
//! #[hana_function()]
//! fn succ(i: Value::Int) {
//!     Value::Int(i + 1)
//! }
//! ```
//!
//! The macro should generate a function with the signature:
//! ```rust,text
//! pub extern "C" fn succ(vm: *const Vm, nargs: u16) {
//!     ...
//! }
//! ```

extern crate proc_macro;
#[macro_use]
extern crate quote;
use proc_macro::TokenStream;
use syn::{self, spanned::Spanned};



/// Generates a native function callable from haru's virtual machine.
///
/// Note that the file containing your native function must contain
/// the imports for Vm and Value:
///
/// ```rust,text
/// extern crate haru;
/// use haru::vmbindings::value::Value;
/// use haru::vmbindings::vm::Vm;
/// ```
///
/// Example:
///
/// ```rust,text
/// #[hana_function()]
/// fn fopen(path: Value::String, mode: Value::String) {
///     [body of fopen]
/// }
/// ```
///
/// should generate a function like this (semi pseudocode):
///
/// ```rust,text
/// pub extern "C" fn fopen(cvm : *mut Vm, nargs : u16) {
///     if nargs != [nargs] { [raise vm error] }
///     fn fopen() -> Value {
///         let Value::String(path) = vm.stack.pop().unwrap() ||
///                 panic!("expected path to be string");
///         let Value::String(mode) = vm.stack.pop().unwrap() ||
///                 panic!("expected mode to be string");
///         [body of fopen]
///     }
///     let vm = unsafe { &mut *cvm };
///     let result : Value = #name(vm);
///     vm.stack.push(result.wrap());
/// }
/// ```
#[proc_macro_attribute]
pub fn hana_function(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let name = &input.sig.ident;
    let body = &input.block;

    let mut args_setup = Vec::new();
    // deprecated code: decl.inputs.iter() {
    for arg in input.sig.inputs.iter() {
        match *arg {
            syn::FnArg::Typed(ref cap) => {
                let pattern = match *cap.pat.clone() {
                    syn::Pat::Ident(x) => x,
                    _ => panic!("expected identifier argument!"),
                };
                let path = match *cap.ty.clone() {
                    syn::Type::Path(x) => x.path.segments,
                    _ => panic!("expected type for {:?} to be path!", pattern),
                };
                // match and unwrap type from value variant
                // also panics if unexpected type
                let atype = path.last().unwrap().ident.to_string();
                //.into_value().ident.to_string();
                let atypes = syn::LitStr::new(atype.as_str(), atype.span()); // quote::__rt::Span::call_site());
                let argname = syn::LitStr::new(
                    pattern.ident.to_string().as_str(),
                    pattern.ident.span(),
                    //quote::__rt::Span::call_site(),
                );
                let match_arm = match atype.as_str() {
                    "Int" | "Float" | "NativeFn" | "Fn" | "Str" | "Record" | "Array" => {
                        quote!(#path(x) => x)
                    }
                    "Any" => quote!(#path => x),
                    _ => panic!("unknown type {}!", atype),
                };
                args_setup.push(match atype.as_str() {
                    "Any" => quote!(let #pattern = vm.borrow_mut().stack.pop().unwrap() ;),
                    _ => quote!(
                        let #pattern = {
                            match  vm.borrow_mut().stack.pop().unwrap()  {
                                #match_arm,
                                _ => panic!("expected argument {} to be type {}",
                                    #argname,
                                    #atypes)
                            }
                        };
                    ),
                });
            }
            _ => unimplemented!(),
        }
    }

    let arglen = syn::LitInt::new(
        input.sig.inputs.len().to_string().as_str(),
        input.sig.inputs.span(),
    );

    quote!(
        pub fn #name(vm: std::rc::Rc<std::cell::RefCell<Vm>>, nargs : u16) {

            if nargs != #arglen {
                use super::VmError;
                vm.borrow_mut().error = VmError::ERROR_MISMATCH_ARGUMENTS;
                vm.borrow_mut().error_expected = #arglen;
                return;
            }

            #[inline(always)]
            fn #name(vm: std::rc::Rc<std::cell::RefCell<Vm>>) -> Value {
                #(#args_setup)*
                #body
            }

            let result = #name(std::rc::Rc::clone(&vm));
            match result {
                Value::PropagateError => (),
                _ => unsafe{ vm.borrow_mut().stack_push_gray(result) },
            }
        }
    )
    .into()
}
