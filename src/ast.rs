//! Provides parser and abstract syntax tree structures.
//!
//! The parser exports the function `grammar::start`
//! used to generate a vector of abstract syntax trees
//! representing statements. The Ast can then be used to
//! emit raw bytecode to a `Compiler`.
//!
//! ```
//! use haru::grammar;
//! let prog = grammar::parser_start("print('Hello World')").unwrap();
//! ```

// Provides abstract syntax trees for language blocks.
use crate::compiler;
use crate::vmbindings::vm::VmOpcode;
use std::any::Any;
use std::fmt;

// #region macros
macro_rules! ast_impl {
    () => {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn span(&self) -> &Span {
            &self._span
        }
    };
}

macro_rules! emit_begin {
    ($self:ident, $c:ident) => {{
        let mut modules_info = $c.modules_info.borrow_mut();
        let len = modules_info.files.len();
        modules_info.smap.push(compiler::SourceMap {
            file: $self.span().clone(),
            fileno: if len == 0 { 0 } else { len - 1 },
            bytecode: ($c.clen(), 0),
        });
    }};
}

macro_rules! smap_begin {
    ($c:ident) => {
        $c.modules_info.borrow().smap.len() - 1
    };
}

macro_rules! emit_end {
    ($c:ident, $smap:expr) => {
        $c.modules_info.borrow_mut().smap[$smap].bytecode.1 = $c.clen();
    };
}

macro_rules! try_nil {
    ($e:expr) => {
        if let Err(_) = $e {
            return Err(CodeGenError::NilString);
        }
    };
}

macro_rules! op_push_str {
    ($c:ident, $s:expr) => {
        if let Some(interned_strings) = $c.interned_strings.as_mut() {
            if let Some(idx) = interned_strings.get_or_insert(&$s) {
                $c.cpushop(VmOpcode::PushStrInterned);
                $c.cpush16(idx);
            } else {
                $c.cpushop(VmOpcode::PushStr);
                try_nil!($c.cpushs($s.clone()));
            }
        } else {
            $c.cpushop(VmOpcode::PushStr);
            try_nil!($c.cpushs($s.clone()));
        }
    };
}
// #endregion

/// Code generation result
#[derive(Debug)]
pub enum CodeGenError {
    InvalidLeftHandSide,
    ExpectedIdentifier,
    ExpectedInFunction,
    NilString,
}
pub type CodeGenResult = Result<(), CodeGenError>;

/// Span of the Asrt node, represented by a tuple of (from, to) indexes
pub type Span = (usize, usize);
// Byte range in the source.
//pub type Span = core::ops::Range<usize>;
/// Generic abstract syntax tree
pub trait Ast: fmt::Debug {
    fn as_any(&self) -> &dyn Any;
    fn span(&self) -> &Span;
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult;
}

// # values
/// Identifier node
pub struct Identifier {
    pub _span: Span,
    pub val: String,
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"identifier\": \"{}\"}}", self.val)
    }
}
impl Ast for Identifier {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.emit_get_var(self.val.clone());
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// String literal
pub struct StrLiteral {
    pub _span: Span,
    pub val: String,
}

impl fmt::Debug for StrLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"string\": {:?}}}", self.val)
    }
}
impl Ast for StrLiteral {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        op_push_str!(c, self.val);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Integer literal
pub struct IntLiteral {
    pub _span: Span,
    pub val: i64,
}

impl fmt::Debug for IntLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Integer {{ value: {} }}", self.val)
    }
}
impl Ast for IntLiteral {
    ast_impl!();

    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        let n = self.val as u64;
        match n {
            0..=0xff => {
                c.cpushop(VmOpcode::Push8);
                c.cpush8(n as u8);
            }
            0x100..=0xffff => {
                c.cpushop(VmOpcode::Push16);
                c.cpush16(n as u16);
            }
            0x10000..=0xffffffff => {
                c.cpushop(VmOpcode::Push32);
                c.cpush32(n as u32);
            }
            _ => {
                c.cpushop(VmOpcode::Push64);
                c.cpush64(n);
            }
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Float literal
pub struct FloatLiteral {
    pub _span: Span,
    pub val: f64,
}

impl fmt::Debug for FloatLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"float\": {}}}", self.val)
    }
}
impl Ast for FloatLiteral {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.cpushop(VmOpcode::Pushf64);
        c.cpushf64(self.val);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Array literals
pub struct ArrayExpr {
    pub _span: Span,
    pub exprs: Vec<Box<dyn Ast>>,
}

impl fmt::Debug for ArrayExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"array\": {:?}}}", self.exprs)
    }
}
impl Ast for ArrayExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        for expr in &self.exprs {
            expr.emit(c)?;
        }
        if self.exprs.len() < 0x100 {
            c.cpushop(VmOpcode::Push8);
            c.cpush8(self.exprs.len() as u8);
        } else {
            c.cpushop(VmOpcode::Push64);
            c.cpush64(self.exprs.len() as u64);
        }
        c.cpushop(VmOpcode::ArrayLoad);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Function expression
pub struct FunctionDefinition {
    pub _span: Span,
    pub id: Option<String>,
    pub args: Vec<String>,
    pub stmt: Box<dyn Ast>,
}

impl fmt::Debug for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut args: String = "[".to_string();
        let argsv: Vec<String> = self.args.iter().map(|x| format!("\"{}\"", x)).collect();
        args += &argsv.join(",");
        args += "]";
        write!(
            f,
            "
                id: {:?},
                args: {},
                stmt: {:?}",
            self.id.as_ref().map_or("".to_string(), |x| x.clone()),
            args,
            self.stmt
        )
    }
}
impl Ast for FunctionDefinition {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        // definition
        c.cpushop(VmOpcode::DefFunctionPush);
        c.cpush16(self.args.len() as u16);
        let function_end = c.reserve_label16();

        if let Some(id) = &self.id {
            c.set_local(id.clone());
        }
        c.scope();

        // body
        c.cpushop(VmOpcode::EnvNew);
        let nslot_label = c.reserve_label16();
        for arg in &self.args {
            c.set_local(arg.clone());
        }
        self.stmt.emit(c)?;
        if let Some(id) = &self.id {
            let len = c.clen() - 1;
            let mut modules_info = c.modules_info.borrow_mut();
            modules_info.symbol.insert(len, id.clone());
        }

        // default return
        // WARNING:  This has not been proven to work properly.
        let u = unsafe {
            let u: *const VmOpcode = c.ctop() as *const VmOpcode;
            *u
        };

        match u {
            VmOpcode::Ret => {}
            VmOpcode::RetCall => {}
            _ => {
                c.cpushop(VmOpcode::PushNil);
                c.cpushop(VmOpcode::Ret);
            }
        };

        // end
        let nslots = c.unscope();
        c.fill_label16(nslot_label, nslots);
        c.fill_label16(function_end, (c.clen() - function_end) as u16);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Record expression
pub struct RecordDefinition {
    pub _span: Span,
    pub id: Option<String>,
    pub stmts: Vec<Box<dyn Ast>>,
}

impl fmt::Debug for RecordDefinition {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for RecordDefinition {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        for stmt in &self.stmts {
            let any = stmt.as_any();
            if let Some(stmt) = any.downcast_ref::<FunctionStatement>() {
                stmt.def().emit(c)?;
                op_push_str!(c, stmt.def().id.as_ref().unwrap());
            } else if let Some(stmt) = any.downcast_ref::<RecordStatement>() {
                stmt.def().emit(c)?;
                op_push_str!(c, stmt.def().id.as_ref().unwrap());
            } else if let Some(stmt) = any.downcast_ref::<ExprStatement>() {
                let binexpr = stmt.expr.as_any().downcast_ref::<BinExpr>().unwrap();
                let id = if let Some(id) = binexpr.left.as_any().downcast_ref::<Identifier>() {
                    id
                } else {
                    return Err(CodeGenError::InvalidLeftHandSide);
                };
                binexpr.right.emit(c)?;
                op_push_str!(c, id.val);
            }
        }
        if self.stmts.len() < 0x100 {
            c.cpushop(VmOpcode::Push8);
            c.cpush8(self.stmts.len() as u8);
        } else {
            c.cpushop(VmOpcode::Push64);
            c.cpush64(self.stmts.len() as u64);
        }
        c.cpushop(VmOpcode::DictLoad);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

// # expressions
/// Unary operations
pub enum UnaryOp {
    Not,
    Neg,
}
/// Unary expressions
pub struct UnaryExpr {
    pub _span: Span,
    pub op: UnaryOp,
    pub val: Box<dyn Ast>,
}

impl fmt::Debug for UnaryExpr {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for UnaryExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        self.val.emit(c)?;
        match self.op {
            UnaryOp::Not => {
                c.cpushop(VmOpcode::Not);
            }
            UnaryOp::Neg => {
                c.cpushop(VmOpcode::Negate);
            }
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Conditional expressions
pub struct CondExpr {
    pub _span: Span,
    pub cond: Box<dyn Ast>,
    pub then: Box<dyn Ast>,
    pub alt: Box<dyn Ast>,
}
impl CondExpr {
    fn _emit(&self, c: &mut compiler::Compiler, is_tail: bool) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        // Pseudo code of the generated bytecode
        //   [condition]
        //   jncond [else]
        //   [statement]
        //   jmp done
        //   [else]
        //   [done]
        self.cond.emit(c)?;
        c.cpushop(VmOpcode::JNcond); // TODO: maybe do peephole opt?
        let else_label = c.reserve_label16();

        if is_tail {
            if let Some(expr) = self.then.as_any().downcast_ref::<CallExpr>() {
                expr._emit(c, true)?;
            } else if let Some(expr) = self.then.as_any().downcast_ref::<CondExpr>() {
                expr._emit(c, true)?;
            } else {
                self.then.emit(c)?;
            }
        } else {
            self.then.emit(c)?;
        }

        c.cpushop(VmOpcode::Jmp);
        let done_label = c.reserve_label16();
        c.fill_label16(else_label, (c.clen() - else_label) as u16);

        if is_tail {
            if let Some(expr) = self.alt.as_any().downcast_ref::<CallExpr>() {
                expr._emit(c, true)?;
            } else if let Some(expr) = self.then.as_any().downcast_ref::<CondExpr>() {
                expr._emit(c, true)?;
            } else {
                self.alt.emit(c)?;
            }
        } else {
            self.alt.emit(c)?;
        }

        c.fill_label16(done_label, (c.clen() - done_label) as u16);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

impl fmt::Debug for CondExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "cond: {:?}, then: {:?}, alt: {:?}, op: cond",
            self.cond, self.then, self.alt
        )
    }
}
impl Ast for CondExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self._emit(c, false)
    }
}
/// Binary operators
#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Neq,
    Gt,
    Lt,
    Geq,
    Leq,
    Assign,
    Adds,
    Subs,
    Muls,
    Divs,
    Mods,
    Of,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
}
/// Binary expressions
pub struct BinExpr {
    pub _span: Span,
    pub left: Box<dyn Ast>,
    pub right: Box<dyn Ast>,
    pub op: BinOp,
}

impl fmt::Debug for BinExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{\"left\": {:?}, \"right\": {:?}, \"op\": \"{}\"}}",
            self.left,
            self.right,
            match &self.op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                BinOp::And => "and",
                BinOp::Or => "or",
                BinOp::Eq => "==",
                BinOp::Neq => "!=",
                BinOp::Gt => ">",
                BinOp::Geq => ">=",
                BinOp::Lt => "<",
                BinOp::Leq => "<=",
                BinOp::Assign => "=",
                BinOp::Adds => "+=",
                BinOp::Subs => "-=",
                BinOp::Muls => "*=",
                BinOp::Divs => "/=",
                BinOp::Mods => "%=",
                BinOp::Of => "of",
                BinOp::BitwiseAnd => "&",
                BinOp::BitwiseOr => "|",
                BinOp::BitwiseXor => "xor",
            }
        )
    }
}
impl Ast for BinExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        macro_rules! arithop_do {
            ($x:expr) => {{
                self.left.emit(c)?;
                self.right.emit(c)?;
                c.cpushop($x);
            }};
        }
        match self.op {
            // assignment
            BinOp::Assign => {
                let any = self.left.as_any();
                if let Some(id) = any.downcast_ref::<Identifier>() {
                    self.right.emit(c)?;
                    c.emit_set_var(id.val.clone(), false);
                } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                    self.right.emit(c)?;
                    memexpr._emit(c, MemExprEmit::SetOp)?;
                } else if let Some(callexpr) = any.downcast_ref::<CallExpr>() {
                    // definition
                    c.cpushop(VmOpcode::DefFunctionPush);
                    c.cpush16(callexpr.args.len() as u16);
                    let function_end = c.reserve_label16();

                    c.set_local(
                        if let Some(callee) = callexpr.callee.as_any().downcast_ref::<Identifier>()
                        {
                            callee.val.clone()
                        } else {
                            return Err(CodeGenError::ExpectedIdentifier);
                        },
                    );
                    c.scope();

                    // body
                    c.cpushop(VmOpcode::EnvNew);
                    let nslot_label = c.reserve_label16();
                    for arg in &callexpr.args {
                        c.set_local(
                            if let Some(arg) = arg.as_any().downcast_ref::<Identifier>() {
                                arg.val.clone()
                            } else {
                                return Err(CodeGenError::ExpectedIdentifier);
                            },
                        );
                    }

                    if let Some(expr) = self.right.as_any().downcast_ref::<CallExpr>() {
                        expr._emit(c, true)?;
                    } else if let Some(expr) = self.right.as_any().downcast_ref::<CondExpr>() {
                        expr._emit(c, true)?;
                        c.cpushop(VmOpcode::Ret);
                    } else {
                        self.right.emit(c)?;
                        c.cpushop(VmOpcode::Ret);
                    }

                    // end
                    let nslots = c.unscope();
                    c.fill_label16(nslot_label, nslots);
                    c.fill_label16(function_end, (c.clen() - function_end) as u16);

                    let id =
                        if let Some(id) = &callexpr.callee.as_any().downcast_ref::<Identifier>() {
                            id.val.clone()
                        } else {
                            return Err(CodeGenError::ExpectedIdentifier);
                        };
                    if id != "_" {
                        // _ for id is considered a anonymous function decl
                        c.emit_set_var(id, true);
                    }
                } else {
                    return Err(CodeGenError::InvalidLeftHandSide);
                }
            }
            BinOp::Adds | BinOp::Subs | BinOp::Muls | BinOp::Divs | BinOp::Mods => {
                let opcode = match self.op {
                    BinOp::Adds => VmOpcode::IAdd,
                    BinOp::Subs => VmOpcode::Sub, // OP_ISUB?
                    BinOp::Muls => VmOpcode::IMul,
                    BinOp::Divs => VmOpcode::Div, // OP_IDIV?
                    BinOp::Mods => VmOpcode::Mod, // OP_IMOD?
                    _ => unreachable!(),
                };
                let any = self.left.as_any();
                //let mut in_place_addr = std::usize::MAX;
                if let Some(id) = any.downcast_ref::<Identifier>() {
                    c.emit_get_var(id.val.clone());
                    self.right.emit(c)?;
                    c.cpushop(opcode);
                    /*
                    match opcode {
                        VmOpcode::IADD | VmOpcode::IMUL => {
                            in_place_addr = c.clen();
                            c.cpush8(0);
                        }
                        _ => {}
                    };
                    */
                    c.emit_set_var(id.val.clone(), false);
                } else if let Some(memexpr) = any.downcast_ref::<MemExpr>() {
                    memexpr.left.emit(c)?;

                    // optimize static member vars
                    let val = {
                        let any = memexpr.right.as_any();
                        any.downcast_ref::<Identifier>()
                            .map(|str| &str.val)
                            .or_else(|| any.downcast_ref::<StrLiteral>().map(|str| &str.val))
                    };

                    // prologue
                    if let (Some(value), false) = (val, memexpr.is_expr) {
                        c.cpushop(VmOpcode::MemberGetNoPop);
                        try_nil!(c.cpushs(value.clone()));
                    } else {
                        memexpr.right.emit(c)?;
                        c.cpushop(VmOpcode::IndexGetNoPop);
                    }

                    // body
                    self.right.emit(c)?;
                    c.cpushop(opcode);

                    /*
                    match opcode {
                        VmOpcode::IADD | VmOpcode::IMUL => {
                            in_place_addr = c.clen();
                            c.cpush8(0);
                        }
                        _ => {}
                    };
                    // epilogue
                    if in_place_addr != std::usize::MAX {
                        // jmp here if we can do it in place
                        let len = c.clen();
                        c.cfill_label8(in_place_addr, (len - in_place_addr) as u8);
                    }*/

                    c.cpushop(VmOpcode::Swap);
                    if let (Some(value), false) = (val, memexpr.is_expr) {
                        c.cpushop(VmOpcode::MemberSet);
                        try_nil!(c.cpushs(value.clone()));
                    } else {
                        // otherwise, do OP_INDEX_SET as normal
                        memexpr.right.emit(c)?;
                        c.cpushop(VmOpcode::IndexSet);
                    }
                    emit_end!(c, _smap_begin);
                    return Ok(());
                } else {
                    return Err(CodeGenError::InvalidLeftHandSide);
                }
                /*
                if in_place_addr != std::usize::MAX {
                // jmp here if we can do it in place
                let len = c.clen();
                c.cfill_label8(in_place_addr, (len - in_place_addr) as
                 u8);
                }
                */
            }
            // basic manip operators
            BinOp::Add => arithop_do!(VmOpcode::Add),
            BinOp::Sub => arithop_do!(VmOpcode::Sub),
            BinOp::Mul => arithop_do!(VmOpcode::Mul),
            BinOp::Div => arithop_do!(VmOpcode::Div),
            BinOp::Mod => arithop_do!(VmOpcode::Mod),
            BinOp::Eq => arithop_do!(VmOpcode::Eq),
            BinOp::Neq => arithop_do!(VmOpcode::NEq),
            BinOp::Gt => arithop_do!(VmOpcode::Gt),
            BinOp::Lt => arithop_do!(VmOpcode::Lt),
            BinOp::Geq => arithop_do!(VmOpcode::GEq),
            BinOp::Leq => arithop_do!(VmOpcode::LEq),
            BinOp::Of => arithop_do!(VmOpcode::Of),
            BinOp::BitwiseAnd => arithop_do!(VmOpcode::BitwiseAnd),
            BinOp::BitwiseOr => arithop_do!(VmOpcode::BitwiseOr),
            BinOp::BitwiseXor => arithop_do!(VmOpcode::BitwiseXOR),
            // boolean operators
            BinOp::And => {
                self.left.emit(c)?;
                c.cpushop(VmOpcode::JNcondNoPop);
                let label = c.reserve_label16();
                c.cpushop(VmOpcode::Pop);
                self.right.emit(c)?;
                c.fill_label16(label, (c.clen() - label) as u16);
            }
            BinOp::Or => {
                self.left.emit(c)?;
                c.cpushop(VmOpcode::JCondNoPop);
                let label = c.reserve_label16();
                c.cpushop(VmOpcode::Pop);
                self.right.emit(c)?;
                c.fill_label16(label, (c.clen() - label) as u16);
            } //_ => panic!("not implemented: {:?}", self.op)
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// Member expressions
pub struct MemExpr {
    pub _span: Span,
    pub left: Box<dyn Ast>,
    pub right: Box<dyn Ast>,
    pub is_expr: bool,
    pub is_namespace: bool,
}
#[derive(PartialEq)]
enum MemExprEmit {
    Default,
    MethodCall,
    SetOp,
}
impl MemExpr {
    fn _emit(&self, c: &mut compiler::Compiler, emit_type: MemExprEmit) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        self.left.emit(c)?;
        let any = self.right.as_any();

        // optimize static keys
        let val = {
            any.downcast_ref::<Identifier>()
                .and_then(|id| if !self.is_expr { Some(&id.val) } else { None })
                .or_else(|| any.downcast_ref::<StrLiteral>().map(|str| &str.val))
        };
        if emit_type == MemExprEmit::SetOp {
            // optimize if it's a string
            if let Some(val) = val {
                // optimize if it's interned
                if let Some(interned_strings) = c.interned_strings.as_mut() {
                    if let Some(idx) = interned_strings.get_or_insert(&val) {
                        c.cpushop(VmOpcode::PushStrInterned);
                        c.cpush16(idx);
                        c.cpushop(VmOpcode::IndexSet);

                        emit_end!(c, _smap_begin);
                        return Ok(());
                    }
                }
                // or optimize statically
                c.cpushop(VmOpcode::MemberSet);
                try_nil!(c.cpushs(val.clone()));
            } else {
                // do it normally
                self.right.emit(c)?;
                c.cpushop(VmOpcode::IndexSet);
            }
        } else if val.is_some() && !self.is_expr {
            c.cpushop(if emit_type == MemExprEmit::MethodCall {
                VmOpcode::MemberGetNoPop
            } else {
                VmOpcode::MemberGet
            });
            try_nil!(c.cpushs(val.unwrap().clone()));
        } else {
            self.right.emit(c)?;
            c.cpushop(if emit_type == MemExprEmit::MethodCall {
                VmOpcode::IndexGetNoPop
            } else {
                VmOpcode::IndexGet
            });
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

impl fmt::Debug for MemExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "left: {:?}, right: {:?}, op: memexpr",
            self.left, self.right
        )
    }
}
impl Ast for MemExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self._emit(c, MemExprEmit::Default)
    }
}

/// Call expressions
pub struct CallExpr {
    pub _span: Span,
    pub callee: Box<dyn Ast>,
    pub args: Vec<Box<dyn Ast>>,
}
impl CallExpr {
    fn _emit(&self, c: &mut compiler::Compiler, is_tail: bool) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        let op = if is_tail {
            VmOpcode::RetCall
        } else {
            VmOpcode::Call
        };
        for arg in self.args.iter().rev() {
            arg.emit(c)?;
        }
        if let Some(memexpr) = self.callee.as_any().downcast_ref::<MemExpr>() {
            let _right = memexpr.right.as_any();
            if memexpr.is_namespace {
                memexpr._emit(c, MemExprEmit::Default)?;
                c.cpushop(op);
                c.cpush16(self.args.len() as u16);
            } else {
                memexpr._emit(c, MemExprEmit::MethodCall)?;
                c.cpushop(op);
                c.cpush16((self.args.len() as u16) + 1);
            }
        } else {
            self.callee.emit(c)?;
            c.cpushop(op);
            c.cpush16(self.args.len() as u16);
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

impl fmt::Debug for CallExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{\"callee\": {:?}, \"args\": {:?}, \"op\": \"call\"}}",
            self.callee, self.args
        )
    }
}
impl Ast for CallExpr {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self._emit(c, false)
    }
}
/// Call expression arm (for internal usage)
pub enum CallExprArm {
    MemExprIden(Box<dyn Ast>),
    MemExprNs(Box<dyn Ast>),
    MemExpr(Box<dyn Ast>),
    CallExpr(Vec<Box<dyn Ast>>),
}

// #region statement
// ## control flows
/// If statements
pub struct IfStatement {
    pub _span: Span,
    pub expr: Box<dyn Ast>,
    pub then: Box<dyn Ast>,
    pub alt: Option<Box<dyn Ast>>,
}

impl fmt::Debug for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const FMT_END: &str = "\"type\": \"ifstmt\"";
        match &self.alt {
            Some(alt) => write!(
                f,
                "{{\"expr\": {:?}, \"then\": {:?}, \"alt\": {:?}, {}}}",
                self.expr, self.then, alt, FMT_END
            ),
            None => write!(
                f,
                "{{\"expr\": {:?}, \"then\": {:?}, {}}}",
                self.expr, self.then, FMT_END
            ),
        }
    }
}
impl Ast for IfStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        // Pseudo code of the generated bytecode
        //   [condition]
        //   jncond [else]
        //   [statement]
        //   jmp done
        //   [else]
        //   [done]
        self.expr.emit(c)?;
        c.cpushop(VmOpcode::JNcond); // TODO: maybe do peephole opt?
        let else_label = c.reserve_label16();
        self.then.emit(c)?;
        if let Some(alt) = &self.alt {
            c.cpushop(VmOpcode::Jmp);
            let done_label = c.reserve_label16();
            c.fill_label16(else_label, (c.clen() as isize - else_label as isize) as u16);
            alt.emit(c)?;
            c.fill_label16(done_label, (c.clen() - done_label) as u16);
        } else {
            c.fill_label16(else_label, (c.clen() as isize - else_label as isize) as u16);
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// While statements
pub struct WhileStatement {
    pub _span: Span,
    pub expr: Box<dyn Ast>,
    pub then: Box<dyn Ast>,
}

impl fmt::Debug for WhileStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{\"expr\": {:?}, \"then\": {:?}, \"type\": \"whilestmt\"}}",
            self.expr, self.then
        )
    }
}
impl Ast for WhileStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        // pseudocode of generated bytecode:
        //   begin: jmp condition
        //   [statement]
        //   [condition]
        //   jcond [begin]
        c.cpushop(VmOpcode::Jmp);
        let begin_label = c.reserve_label16();

        let then_label = c.clen();
        c.loop_start();
        self.then.emit(c)?;

        c.fill_label16(begin_label, (c.clen() - begin_label) as u16);

        let next_it_pos = c.clen();
        self.expr.emit(c)?;
        c.cpushop(VmOpcode::JCond);
        c.cpush16((then_label as isize - c.clen() as isize) as u16);
        c.loop_end(next_it_pos, c.clen());
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// For..in statements
pub struct ForInStatement {
    pub _span: Span,
    pub id: String,
    pub expr: Box<dyn Ast>,
    pub stmt: Box<dyn Ast>,
}

impl fmt::Debug for ForInStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
                id: {:?},
                expr: {:?},
                statement: {:?}",
            self.id, self.expr, self.stmt
        )
    }
}
impl Ast for ForInStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        // TODO: OP_FOR (pushes val onto stack)
        // stack: [int iterator pos]
        // code:
        //  [Push array]
        //  next_it: OP_FOR [end]
        //  set id
        //  [body]
        //  jmp [next_it]
        //  [end]
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);

        self.expr.emit(c)?;
        let next_it_label = c.clen();
        c.cpushop(VmOpcode::ForIn);
        let end_label = c.reserve_label16();
        c.emit_set_var(self.id.clone(), false);
        c.cpushop(VmOpcode::Pop);
        self.stmt.emit(c)?;
        c.cpushop(VmOpcode::Jmp);
        c.cpush16((next_it_label as isize - c.clen() as isize) as u16);
        c.fill_label16(end_label, (c.clen() - end_label) as u16);

        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Continue statements
pub struct ContinueStatement {
    pub _span: Span,
}

impl fmt::Debug for ContinueStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for ContinueStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.cpushop(VmOpcode::Jmp);
        c.loop_continue();
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Break statement
pub struct BreakStatement {
    pub _span: Span,
}

impl fmt::Debug for BreakStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for BreakStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.cpushop(VmOpcode::Jmp);
        c.loop_break();
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

// ## other
/// Function definition statement
pub struct FunctionStatement {
    pub _span: Span,
    def: FunctionDefinition,
}
impl FunctionStatement {
    pub fn new(def: FunctionDefinition, span: Span) -> FunctionStatement {
        FunctionStatement { _span: span, def }
    }

    pub fn def(&self) -> &FunctionDefinition {
        &self.def
    }
}

impl fmt::Debug for FunctionStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for FunctionStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self.def.emit(c)?;

        // set var
        c.emit_set_var(self.def.id.as_ref().unwrap().clone(), true);
        c.cpushop(VmOpcode::Pop);
        Ok(())
    }
}
/// Return statement
pub struct ReturnStatement {
    pub _span: Span,
    pub expr: Option<Box<dyn Ast>>,
}

impl fmt::Debug for ReturnStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for ReturnStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        if !c.is_in_function() {
            return Err(CodeGenError::ExpectedInFunction);
        }
        match &self.expr {
            Some(expr) => {
                if let Some(expr) = expr.as_any().downcast_ref::<CallExpr>() {
                    expr._emit(c, true)?;
                } else if let Some(expr) = expr.as_any().downcast_ref::<CondExpr>() {
                    expr._emit(c, true)?;
                } else {
                    expr.emit(c)?;
                }
            }
            None => c.cpushop(VmOpcode::PushNil),
        }
        c.cpushop(VmOpcode::Ret);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// Record definition statement
pub struct RecordStatement {
    pub _span: Span,
    def: RecordDefinition,
}
impl RecordStatement {
    pub fn new(def: RecordDefinition, span: Span) -> RecordStatement {
        RecordStatement { _span: span, def }
    }

    pub fn def(&self) -> &RecordDefinition {
        &self.def
    }
}

impl fmt::Debug for RecordStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for RecordStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self.def.emit(c)?;

        // set var
        c.emit_set_var(self.def.id.as_ref().unwrap().clone(), false);
        c.cpushop(VmOpcode::Pop);
        Ok(())
    }
}

/// Try statement
pub struct TryStatement {
    pub _span: Span,
    pub stmts: Vec<Box<dyn Ast>>,
    #[allow(clippy::vec_box)]
    pub cases: Vec<Box<CaseStatement>>,
}

impl fmt::Debug for TryStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for TryStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.cpushop(VmOpcode::PushNil);
        let mut cases_to_fill: Vec<usize> = Vec::new();
        for case in &self.cases {
            // function will take in 1 arg if id is set
            c.cpushop(VmOpcode::DefFunctionPush);
            c.cpush16(if case.id.is_some() { 1 } else { 0 });
            let body_start = c.reserve_label16();
            // id
            if let Some(id) = &case.id {
                let id = id
                    .as_any()
                    .downcast_ref::<Identifier>()
                    .unwrap()
                    .val
                    .clone();
                c.emit_set_var(id, false);
                c.cpushop(VmOpcode::Pop);
            }
            // body
            for s in &case.stmts {
                s.emit(c)?;
            }
            c.cpushop(VmOpcode::ExframeRet);
            cases_to_fill.push(c.reserve_label16());
            // end
            c.fill_label16(body_start, (c.clen() - body_start) as u16);
            // exception type
            case.etype.emit(c)?;
        }
        c.cpushop(VmOpcode::Try);
        for s in &self.stmts {
            s.emit(c)?;
        }
        for hole in cases_to_fill {
            c.fill_label16(hole, (c.clen() - hole) as u16);
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
/// Case statement
pub struct CaseStatement {
    pub _span: Span,
    pub etype: Box<dyn Ast>,
    pub id: Option<Box<dyn Ast>>,
    pub stmts: Vec<Box<dyn Ast>>,
}

impl fmt::Debug for CaseStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for CaseStatement {
    ast_impl!();
    fn emit(&self, _: &mut compiler::Compiler) -> CodeGenResult {
        // this is already generated by try statement
        unreachable!()
    }
}
/// Exception raise statement
pub struct RaiseStatement {
    pub _span: Span,
    pub expr: Box<dyn Ast>,
}

impl fmt::Debug for RaiseStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for RaiseStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        self.expr.emit(c)?;
        c.cpushop(VmOpcode::Raise);
        Ok(())
    }
}

/// Expression statement
pub struct ExprStatement {
    pub _span: Span,
    pub expr: Box<dyn Ast>,
}

impl fmt::Debug for ExprStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expr: {:?}, type: exprstmt", self.expr)
    }
}
impl Ast for ExprStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        self.expr.emit(c)?;
        c.cpushop(VmOpcode::Pop);
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// Module use statement
pub struct UseStatement {
    pub _span: Span,
    pub path: String,
}

impl fmt::Debug for UseStatement {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}
impl Ast for UseStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        c.cpushop(VmOpcode::Use);
        try_nil!(c.cpushs(self.path.clone()));
        emit_end!(c, _smap_begin);
        Ok(())
    }
}

/// Block statement
pub struct BlockStatement {
    pub _span: Span,
    pub stmts: Vec<Box<dyn Ast>>,
}

impl fmt::Debug for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{\"stmts\": {:?}, \"type\": \"blockstmt\"}}",
            self.stmts
        )
    }
}
impl Ast for BlockStatement {
    ast_impl!();
    fn emit(&self, c: &mut compiler::Compiler) -> CodeGenResult {
        emit_begin!(self, c);
        let _smap_begin = smap_begin!(c);
        for stmt in &self.stmts {
            stmt.emit(c)?;
        }
        emit_end!(c, _smap_begin);
        Ok(())
    }
}
// #endregionz

/// Converts a index in string to a tuple of (line, col) position information.
pub fn pos_to_line(input: &str, pos: usize) -> (usize, usize) {
    let before = &input[..pos];
    let line = before.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1;
    let col = before.chars().rev().take_while(|&c| c != '\n').count() + 1;
    (line, col)
}
