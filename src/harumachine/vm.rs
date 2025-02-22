//! Provides an interface for the virtual machine

use std::{
    cell::RefCell,
    mem::{transmute, ManuallyDrop},
    path::Path,
    rc::Rc,
};

use super::env::Env;
use super::exframe::ExFrame;
use super::function::Function;
use super::gc::*;
use super::hmap::HaruHashMap;
use super::interned_string_map::InternedStringMap;
use super::record::Record;
use super::string::HaruString;
use super::value::Value;

use super::vmerror::VmError;
use crate::compiler::{Compiler, ModulesInfo};
use crate::hanayo::HanayoCtx;
use crate::harumachine::inside::inside_execute;

const CALL_STACK_SIZE: usize = 512;

#[repr(transparent)]
#[allow(dead_code)]
pub(super) struct ConstNonNull<T: Sized> {
    pub(super) pointer: std::num::NonZeroUsize,
    phantom: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Sized> ConstNonNull<T> {
    pub fn new(pointer: *const T) -> Option<Self> {
        if !pointer.is_null() {
            unsafe {
                Some(ConstNonNull {
                    pointer: transmute::<*const T, std::num::NonZero<usize>>(pointer),
                    phantom: std::marker::PhantomData,
                })
            }
        } else {
            None
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VmOpcode {
    Halt, // 0
    // stack manip
    Push8,
    Push16,
    Push32,
    Push64,
    PushBool,
    PushNil,
    PushStr,
    PushStrInterned,
    Pushf64,
    Pop, // 10
    // arith
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    IAdd,
    IMul,
    // bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXOR, // 20
    // unary
    Negate,
    Not,
    // comparison
    Lt,
    LEq,
    Gt,
    GEq,
    Eq,
    NEq,
    // matching
    Of, /* type matching */
    // variables
    EnvNew, // 30
    SetLocal,
    SetLocalFunctionDef,
    GetLocal,
    GetLocalUp,
    SetGlobal,
    GetGlobal,
    DefFunctionPush,
    // flow control
    Jmp,
    JmpLong,
    JCond, // 40
    JNcond,
    Call,
    Ret,
    JCondNoPop,
    JNcondNoPop,
    // dictionary
    DictNew,
    MemberGet,
    MemberGetNoPop,
    MemberSet,
    DictLoad, // 50
    ArrayLoad,
    IndexGet,
    IndexGetNoPop,
    IndexSet,
    // exceptions
    Try,
    Raise,
    ExframeRet,
    // tail calls
    RetCall,
    // iterators
    ForIn,
    Swap, // 60
    // modules
    Use,
}

impl VmOpcode {
    // NOTE: This variable must be updated if use is no longer the last operator.
    pub const VM_OPCODE_COUNT: u8 = VmOpcode::Use as u8;

    pub fn from_u8(value: u8) -> Option<Self> {
        if value <= Self::VM_OPCODE_COUNT {
            unsafe {
                let value_as_op = transmute::<u8, VmOpcode>(value);
                Some(value_as_op)
            }
        } else {
            None
        }
    }
}

impl PartialEq<u8> for VmOpcode {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}

#[repr(C)]
pub struct Vm {
    pub(super) ip: u32, // current instruction pointer
    // pointer to current stack frame
    pub(super) localenv: Vec<Rc<RefCell<Option<Env>>>>,
    // global environment, all unscoped variables/variables
    // starting with '$' should also be stored here without '$'
    globalenv: Option<Box<HaruHashMap>>,
    exframes: Option<Vec<ExFrame>>, // exception frame
    pub code: Vec<u8>,              // where all the code is
    pub stack: Vec<Value>,          // stack

    // prototype types for primitive values
    pub(crate) dstr: Option<Gc<Record>>,
    pub(crate) dint: Option<Gc<Record>>,
    pub(crate) dfloat: Option<Gc<Record>>,
    pub(crate) darray: Option<Gc<Record>>,
    pub(crate) drec: Option<Gc<Record>>,

    pub error: VmError,
    pub error_expected: u32,

    // for handling exceptions inside of interpreted functions called by native functions
    pub(super) exframe_fallthrough: Option<ExFrame>,
    pub(super) native_call_depth: usize,

    // rust-specific fields
    pub interned_strings: Option<InternedStringMap>,
    pub modules_info: Option<Rc<RefCell<ModulesInfo>>>,
    pub(crate) stdlib: Option<HanayoCtx>,
    gc_manager: Option<RefCell<GcManager>>,
}

use super::inside::vm_call;
impl Vm {
    fn new(
        code: Vec<u8>,
        modules_info: Option<Rc<RefCell<ModulesInfo>>>,
        interned_strings: Option<InternedStringMap>,
    ) -> Vm {
        Vm {
            ip: 0,
            localenv: Vec::with_capacity(CALL_STACK_SIZE),
            globalenv: Some(Box::new(HaruHashMap::new())),
            exframes: Some(Vec::with_capacity(2)),
            code,
            stack: Vec::with_capacity(2),
            dstr: None,
            dint: None,
            dfloat: None,
            darray: None,
            drec: None,
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            exframe_fallthrough: None,
            native_call_depth: 0,
            interned_strings,
            modules_info,
            stdlib: None,
            gc_manager: Some(RefCell::new(GcManager::new())),
        }
    }

    //  // pointer to start of pool of stack frames
    //  pub fn localenv_bp(&self) -> Rc<Option<Env>> {
    //     Rc::clone(&self.localenv)
    //  }

    #[allow(dead_code)]
    /// For Debug propoces
    pub fn print_stack(&self) {
        // TODO: move vm_print_stack here and expose function through C ffi
        eprint!("[");
        for value in &self.stack {
            eprint!("{:?} ", value);
        }
        eprintln!("]");
    }

    // interned string
    pub fn get_interned_string(&self, n: u16) -> HaruString {
        HaruString::new_cow(
            self.interned_strings
                .as_ref()
                .unwrap()
                .get(n)
                .map(Rc::clone)
                .unwrap(),
        )
    }

    // globals
    pub fn global(&self) -> &HaruHashMap {
        use std::borrow::Borrow;
        self.globalenv.as_ref().unwrap().borrow()
    }

    pub fn mut_global(&mut self) -> &mut HaruHashMap {
        use std::borrow::BorrowMut;
        self.globalenv.as_mut().unwrap().borrow_mut()
    }

    // gc
    pub fn malloc<T: Sized + GcTraceable>(&self, val: T) -> Gc<T> {
        self.gc_manager
            .as_ref()
            .unwrap()
            .borrow_mut()
            .malloc(self, val)
    }

    #[allow(dead_code)]
    pub fn gc_disable(&self) {
        self.gc_manager.as_ref().unwrap().borrow_mut().disable()
    }

    pub fn gc_enable(&self) {
        self.gc_manager.as_ref().unwrap().borrow_mut().enable()
    }

    /// # Safety
    ///
    /// This function calls gc (which is unsafe)
    pub unsafe fn stack_push_gray(&mut self, val: Value) {
        if let Some(ptr) = val.as_gc_pointer() {
            self.gc_manager
                .as_ref()
                .unwrap()
                .borrow_mut()
                .push_gray_body(ptr);
        }

        self.stack.push(val);
    }

    // call stack
    // We take a function f, we divert our current ip to the ip of f
    pub fn enter_env(&mut self, fun: &'static Function) {
        if self.localenv.len() > CALL_STACK_SIZE {
            panic!("maximum stack depth exceeded");
        }

        self.localenv.push(Rc::new(RefCell::new(Some(Env::new(
            self.ip,
            fun.get_bound(),
            fun.nargs,
        )))));

        self.ip = fun.ip;
    }

    pub fn enter_env_tail(&mut self, fun: &'static Function) {
        if let Some(localenv) = self.localenv.last() {
            if let Some(mut env) = localenv.take() {
                env.nargs = fun.nargs;
                env.lexical_parent = fun.get_bound();
                *localenv.borrow_mut() = Some(env);
                self.ip = fun.ip;
            }
        }
    }

    pub fn leave_env(&mut self) {
        if let Some(localenv) = self.localenv.pop().take() {
            if let Some(localenv) = localenv.borrow_mut().take() {
                self.ip = localenv.retip;
            }
        }
    }

    // accessors
    pub fn localenv(&self) -> &[Rc<RefCell<Option<Env>>>] {
        &self.localenv[..]
    }

    // exceptions
    fn exframes(&self) -> &Vec<ExFrame> {
        self.exframes.as_ref().unwrap()
    }

    fn mut_exframes(&mut self) -> &mut Vec<ExFrame> {
        self.exframes.as_mut().unwrap()
    }

    pub fn enter_exframe(&mut self) -> &mut ExFrame {
        let localenv = self.localenv.last().map(Rc::clone);
        let len = self.stack.len() - 1;
        let native_call_depth = self.native_call_depth;

        self.mut_exframes()
            .push(ExFrame::new(localenv, len, native_call_depth));
        self.mut_exframes().last_mut().unwrap()
    }

    pub fn leave_exframe(&mut self) {
        self.mut_exframes().pop();
    }

    // execution context for eval
    pub fn new_exec_ctx(&mut self) -> ManuallyDrop<Vm> {
        // prevent context's local variables from being freed

        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            val.ref_inc();
        }
        // call stack
        // if let Some(localenv) = self.localenv {
        //     let mut env = self.localenv_bp;
        //     let localenv = localenv.as_ptr();

        for env in self.localenv.iter() {
            let env_some = env.borrow_mut().take();

            if let Some(ref env) = env_some {
                for val in env.slots.values() {
                    (*val).ref_inc();
                }
            }

            *env.borrow_mut() = env_some;
        }
        

        // save current ctx
        let current_ctx = Vm {
            ip: self.ip,
            localenv: self.localenv.drain(..).collect(),
            globalenv: None, // shared
            exframes: self.exframes.take(),
            code: Vec::new(), // shared
            stack: std::mem::replace(&mut self.stack, Vec::with_capacity(2)),
            // types don't need to be saved:
            dstr: None,
            dint: None,
            dfloat: None,
            darray: None,
            drec: None,
            // shared
            error: VmError::ERROR_NO_ERROR,
            error_expected: 0,
            interned_strings: None,
            exframe_fallthrough: self.exframe_fallthrough.take(),
            native_call_depth: self.native_call_depth,
            modules_info: None,
            stdlib: None,
            gc_manager: None,
        };
        // create new ctx
        self.ip = 0;
        self.exframes = Some(Vec::new());
        ManuallyDrop::new(current_ctx)
    }

    pub fn restore_exec_ctx(&mut self, ctx: ManuallyDrop<Vm>) {
        let mut ctx: Vm = ManuallyDrop::into_inner(ctx);

        self.localenv = Vec::with_capacity(CALL_STACK_SIZE);

        // fill in
        self.ip = ctx.ip;
        self.localenv = ctx.localenv.drain(..).collect();
        self.exframes = ctx.exframes.take();
        self.exframe_fallthrough = ctx.exframe_fallthrough.take();
        self.native_call_depth = ctx.native_call_depth;
        self.stack = std::mem::take(&mut ctx.stack);

        // release context's local variables

        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            val.ref_dec();
        }

        // call stack
        for val in self.localenv.iter() {
            let env_some = val.borrow_mut().take();

            if let Some(ref env) = env_some {
                for val in env.slots.values() {
                    (*val).ref_dec();
                }
            }

            *val.borrow_mut() = env_some;
        }
    }

    // instruction pointer
    pub fn ip(&self) -> u32 {
        self.ip
    }
    pub fn jmp(&mut self, ip: u32) {
        assert!(ip < self.code.len() as u32);
        self.ip = ip;
    }

    // imports
    pub fn load_module(&mut self, path: &str) {
        // loads module, jumps to the module then jump back to OP_USE
        use std::io::Read;

        let rc = self.modules_info.clone().unwrap();

        let pathobj = if path.starts_with("./") {
            let c = rc.borrow_mut();
            let last_path = c.files.last().unwrap();
            let curpath = Path::new(&last_path);
            let mut pathobj = if let Some(parent) = curpath.parent() {
                parent.join(Path::new(path))
            } else {
                Path::new(path).to_path_buf()
            };
            if !pathobj.as_path().is_file() && pathobj.extension().is_none() {
                pathobj.set_extension("hana");
            }
            pathobj
        } else if path.starts_with('/') {
            let mut pathobj = Path::new(path).to_path_buf();
            if !pathobj.as_path().is_file() && pathobj.extension().is_none() {
                pathobj.set_extension("hana");
            }
            pathobj
        } else {
            use std::env;
            match env::var_os("HANA_PATH") {
                Some(parent) => env::split_paths(&parent)
                    .map(|x| {
                        let mut pathobj = Path::new(&x).join(path);
                        if pathobj.extension().is_none() {
                            pathobj.set_extension("hana");
                        }
                        pathobj
                    })
                    .find(|x| x.as_path().is_file())
                    .unwrap(),
                None => panic!("HANA_PATH not set!"),
            }
        };

        if rc.borrow_mut().modules_loaded.contains(&pathobj) {
            return;
        } else {
            rc.borrow_mut().modules_loaded.insert(pathobj.clone());
        }

        if let Ok(mut file) = std::fs::File::open(pathobj) {
            // WARNING: There is no control of the errors generated when importing.
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            let prog = crate::grammar::parser_start(&s).unwrap();
            rc.borrow_mut().files.push(path.to_string());
            rc.borrow_mut().sources.push(s);

            let importer_ip = self.ip;
            let imported_ip = self.code.len();
            {
                let mut c = Compiler::new_append(
                    self.code.clone(),
                    rc,
                    // TODO(xyz): error ↓
                    if self.interned_strings.is_some() {
                        self.interned_strings.take().unwrap()
                    } else {
                        InternedStringMap::new()
                    },
                );
                for stmt in prog {
                    stmt.emit(&mut c).unwrap();
                }
                c.cpushop(VmOpcode::JmpLong);
                c.cpush32(importer_ip);
                self.interned_strings = c.interned_strings.take();
                self.code = c.into_code();
            }
            self.ip = imported_ip as u32;
        }
    }
}

pub fn initialize_vm(
    code: Vec<u8>,
    modules_info: Option<Rc<RefCell<ModulesInfo>>>,
    interned_strings: Option<InternedStringMap>,
) -> Rc<RefCell<Vm>> {
    Rc::new(RefCell::new(Vm::new(code, modules_info, interned_strings)))
}

// functions
pub fn call(vm: Rc<RefCell<Vm>>, fun: Value, args: &[Value]) -> Option<Value> {
    let val = vm_call(vm, fun, args);
    if let Value::InterpreterError = val {
        None
    } else {
        Some(val)
    }
}
// TODO: Use error instance panic
pub fn execute_vm(vm: Rc<RefCell<Vm>>) {
    if vm.borrow().code.is_empty() {
        panic!("calling with nil code");
    }
    //unsafe {}
    inside_execute(vm);
}

pub fn raise(vm: Rc<RefCell<Vm>>) -> bool {
    if vm.borrow().exframes().is_empty() {
        return false;
    }
    let val = vm.borrow().stack.last().unwrap().clone();
    for exframe in vm.borrow().exframes.as_ref().unwrap().iter() {
        if let Some(handler) = exframe.get_handler(Rc::clone(&vm), &val) {
            vm.borrow_mut().ip = handler.ip;
            if handler.nargs == 0 {
                vm.borrow_mut().stack.pop();
            }
            if exframe.unwind_native_call_depth != vm.borrow().native_call_depth {
                vm.borrow_mut().exframe_fallthrough = Some(exframe.clone());
            }
            return true;
        }
    }
    false
}

impl GcTraceable for Vm {
    unsafe fn trace(&self, vec: &mut Vec<*mut GcNode>) {
        for (_, val) in self.global().iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(vec, ptr);
            }
        }
        // stack
        let stack = &self.stack;
        for val in stack.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(vec, ptr);
            }
        }

        // call stack
        for env in self.localenv.iter().filter(|x| x.borrow().is_some()) {
            let env_some = env.borrow_mut().take().unwrap();

            for val in env_some.slots.values() {
                if let Some(ptr) = (*val).as_gc_pointer() {
                    push_gray_body(vec, ptr);
                }
            }

            *env.borrow_mut() = Some(env_some);
        }
    }
}
