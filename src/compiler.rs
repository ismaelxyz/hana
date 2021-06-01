//! Provides a compiler for executing a program
//! and running in the virtual machine.
//!
//! Example for emitting bytecode for the program `print('Hello World')`:
//! ```
//! use haru::{ast, grammar};
//! use haru::compiler::Compiler;
//! use haru::vmbindings::vm::{Vm, VmOpcode};
//! let mut c = Compiler::new(true);
//! let prog = grammar::parser_start("print('Hello World')\n").unwrap();
//! for stmt in prog {
//!     stmt.emit(&mut c);
//! }
//! c.cpushop(VmOpcode::Halt);
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::vmbindings::interned_string_map::InternedStringMap;
use crate::vmbindings::vm::{Vm, VmOpcode};

struct Scope {
    vars: Vec<String>,
}
impl Scope {
    fn new() -> Scope {
        Scope { vars: Vec::new() }
    }
}

struct LoopStatement {
    pub fill_continue: Vec<usize>,
    pub fill_break: Vec<usize>,
}

/// Indexed range for a stream of source code or bytecode.
pub type ArrayIndexRange = (usize, usize);
/// Mapping for a source code range to bytecode range.
#[derive(Clone)]
pub struct SourceMap {
    pub file: ArrayIndexRange,
    pub bytecode: ArrayIndexRange,
    pub fileno: usize,
}

/// Loaded modules info
pub struct ModulesInfo {
    pub smap: Vec<SourceMap>,
    pub files: Vec<String>,
    pub modules_loaded: std::collections::HashSet<std::path::PathBuf>,
    pub symbol: BTreeMap<usize, String>,
    pub sources: Vec<String>,
}

impl ModulesInfo {
    pub fn new() -> ModulesInfo {
        ModulesInfo {
            smap: Vec::new(),
            files: Vec::new(),
            modules_loaded: std::collections::HashSet::new(),
            symbol: BTreeMap::new(),
            sources: Vec::new(),
        }
    }
}

/// Compiler for processing AST nodes and
/// executing generated bytecode in a virtual machine.
pub struct Compiler {
    scopes: Vec<Scope>,
    loop_stmts: Vec<LoopStatement>,
    code: Option<Vec<u8>>,
    pub interned_strings: Option<InternedStringMap>,
    pub modules_info: Rc<RefCell<ModulesInfo>>,
}
impl Compiler {
    pub fn new(interned_strings_enabled: bool) -> Compiler {
        Compiler {
            scopes: Vec::new(),
            loop_stmts: Vec::new(),
            code: Some(Vec::new()),
            interned_strings: if interned_strings_enabled {
                Some(InternedStringMap::new())
            } else {
                None
            },
            modules_info: Rc::new(RefCell::new(ModulesInfo::new())),
        }
    }

    pub fn new_append(
        code: Vec<u8>,
        modules_info: Rc<RefCell<ModulesInfo>>,
        interned_strings: InternedStringMap,
    ) -> Compiler {
        Compiler {
            scopes: Vec::new(),
            loop_stmts: Vec::new(),
            code: Some(code),
            interned_strings: Some(interned_strings),
            modules_info,
        }
    }

    // create
    pub fn into_vm(&mut self) -> Vm {
        Vm::new(
            self.code.clone().take().unwrap(),
            Some(self.modules_info.clone()),
            self.interned_strings.take(),
        )
    }
    pub fn into_code(self) -> Vec<u8> {
        self.code.unwrap()
    }
    pub fn receive_code(&mut self, code: Vec<u8>) {
        self.code = Some(code);
    }
    pub fn take_code(&mut self) -> Vec<u8> {
        self.code.take().unwrap()
    }

    // #region code
    pub fn ctop(&self) -> u8 {
        *self.code.as_ref().unwrap().last().unwrap()
    }
    pub fn clen(&self) -> usize {
        self.code.as_ref().unwrap().len()
    }
    pub fn cpushop(&mut self, n: VmOpcode) {
        self.code.as_mut().unwrap().push(n as u8);
    }
    pub fn cpush8(&mut self, n: u8) {
        self.code.as_mut().unwrap().push(n);
    }
    pub fn cpush16(&mut self, n: u16) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpush32(&mut self, n: u32) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpush64(&mut self, n: u64) {
        for byte in &n.to_be_bytes() {
            self.cpush8(*byte);
        }
    }
    pub fn cpushf64(&mut self, n: f64) {
        let bytes = n.to_bits().to_ne_bytes();
        for byte in &bytes {
            self.cpush8(*byte);
        }
    }
    pub fn cpushs<T: Into<Vec<u8>>>(&mut self, s: T) -> Result<(), ()> {
        let code = self.code.as_mut().unwrap();
        for byte in s.into() {
            if byte == 0 {
                return Err(());
            }
            code.push(byte);
        }
        code.push(0);
        Ok(())
    }

    // labels
    /*
    pub fn cfill_label8(&mut self, pos: usize, label: u8) {
        let code = self.code.as_mut().unwrap();
        code[pos] = label;
    }
    */
    pub fn cfill_label16(&mut self, pos: usize, label: u16) {
        let bytes = label.to_be_bytes();
        let code = self.code.as_mut().unwrap();
        for (i, byte) in bytes.iter().enumerate() {
            code[pos + i] = *byte;
        }
    }
    // other
    pub fn code_as_bytes(&self) -> &[u8] {
        self.code.as_ref().unwrap().as_slice()
    }
    // #endregion

    // scopes
    pub fn is_in_function(&self) -> bool {
        !self.scopes.is_empty()
    }

    // local
    fn get_local(&self, var: &str) -> Option<(u16, u16)> {

        for (relascope, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(slot) = scope.vars.iter().position(|x| *x == *var) {
                return Some((slot as u16, relascope as u16));
            }
        }
        None
    }

    pub fn set_local(&mut self, var: String) -> Option<(u16, u16)> {
        if let Some(last) = self.scopes.last_mut() {
            last.vars.push(var);
            let idx = last.vars.len() - 1;
            return Some((idx as u16, 0));
        }
        None
    }

    // emit set var
    pub fn emit_set_var(&mut self, var: String, is_function: bool) {
        if var.starts_with('$') || self.scopes.is_empty() {
            // set global
            self.cpushop(VmOpcode::SetGlobal);
            self.cpushs(if var.starts_with('$') {
                &var[1..]
            } else {
                var.as_str()
            })
            .unwrap();
        } else if let Some(local) = self.get_local(&var) {
            // set existing local
            let mut slot = local.0;
            let relascope = local.1;
            if relascope != 0 {
                let local = self.set_local(var.clone()).unwrap();
                slot = local.0;
            }
            if is_function {
                self.cpushop(VmOpcode::SetLocalFunctionDef);
                self.cpush16(slot);
            } else {
                self.cpushop(VmOpcode::SetLocal);
                self.cpush16(slot);
            }
        } else {
            let local = self.set_local(var.clone()).unwrap();
            let slot = local.0;
            self.cpushop(VmOpcode::SetLocal);
            self.cpush16(slot);
        }
    }

    pub fn emit_get_var(&mut self, var: String) {
        let local = self.get_local(&var);
        if var.starts_with('$') || local.is_none() {
            // set global
            self.cpushop(VmOpcode::GetGlobal);
            self.cpushs(if var.starts_with('$') {
                &var[1..]
            } else {
                var.as_str()
            })
            .unwrap();
        } else {
            let local = local.unwrap();
            let slot = local.0;
            let relascope = local.1;
            if relascope == 0 {
                self.cpushop(VmOpcode::GetLocal);
                self.cpush16(slot);
            } else {
                self.cpushop(VmOpcode::GetLocalUp);
                self.cpush16(slot);
                self.cpush16(relascope);
            }
        }
    }

    // labels
    pub fn reserve_label16(&mut self) -> usize {
        let pos = self.clen();
        self.cpush16(0);
        pos
    }
    pub fn fill_label16(&mut self, pos: usize, label: u16) {
        self.cfill_label16(pos, label);
    }

    // scopes
    pub fn scope(&mut self) {
        self.scopes.push(Scope::new());
    }
    pub fn unscope(&mut self) -> u16 {
        let size = self.scopes.pop().unwrap().vars.len();
        size as u16
    }

    // loops
    pub fn loop_start(&mut self) {
        self.loop_stmts.push(LoopStatement {
            fill_continue: Vec::new(),
            fill_break: Vec::new(),
        });
    }
    pub fn loop_continue(&mut self) {
        let label = self.reserve_label16();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_continue.push(label);
    }
    pub fn loop_break(&mut self) {
        let label = self.reserve_label16();
        let ls = self.loop_stmts.last_mut().unwrap();
        ls.fill_break.push(label);
    }
    pub fn loop_end(&mut self, next_it_pos: usize, end_pos: usize) {
        let ls = self.loop_stmts.pop().unwrap();
        for label in ls.fill_continue {
            self.fill_label16(label, (next_it_pos - label) as u16);
        }
        for label in ls.fill_break {
            self.fill_label16(label, (end_pos - label) as u16);
        }
    }

    // source map
    pub fn lookup_smap(&self, bc_idx: usize) -> Option<SourceMap> {
        // TODO: fix this and maybe use binary search?
        let mut last_found: Option<SourceMap> = None;
        let modules_info = self.modules_info.borrow();
        for smap in modules_info.smap.iter() {
            if (smap.bytecode.0..=smap.bytecode.1).contains(&bc_idx) {
                // this is so that the lookup gets more "specific"
                last_found = Some((*smap).clone());
            }
        }
        if last_found.is_some() {
            last_found
        } else {
            None
        }
    }
}
