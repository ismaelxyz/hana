use std::io;

use crate::{compiler, grammar, hanayo, vmbindings::vm::VmOpcode};

pub(self) mod errors;
pub(self) mod repl;

use std::io::{Read, Write};

pub struct ParserFlag {
    pub dump_bytecode: bool,
    pub print_ast: bool,
}

// command/file
#[derive(Clone, Eq, PartialEq)]
pub enum ExecutionKind {
    Command(String),
    File(String),
}

pub struct ScriptExecutor {
    arg: ExecutionKind,
    flag: ParserFlag,
    compiler: compiler::Compiler,
    script: String,
}

impl ScriptExecutor {
    pub fn new(arg: ExecutionKind, flag: ParserFlag) -> Self {
        Self {
            arg,
            flag,
            compiler: compiler::Compiler::new(true),
            script: String::new(),
        }
    }

    pub fn load_script(&mut self) {
        self.script = match self.arg.clone() {
            ExecutionKind::Command(cmd) => {
                self.compiler
                    .modules_info
                    .borrow_mut()
                    .files
                    .push("[cmdline]".to_string());
                cmd.to_string()
            }
            ExecutionKind::File(filename) => {
                let mut script = String::new();
                if &filename == "-" {
                    io::stdin()
                        .read_to_string(&mut script)
                        .unwrap_or_else(|err| {
                            println!("error reading from stdin: {}", err);
                            std::process::exit(1);
                        });
                    self.compiler
                        .modules_info
                        .borrow_mut()
                        .files
                        .push("[stdin]".to_string());
                } else {
                    let mut file = std::fs::File::open(&filename).unwrap_or_else(|err| {
                        println!("error opening file: {}", err);
                        std::process::exit(1);
                    });
                    file.read_to_string(&mut script).unwrap_or_else(|err| {
                        println!("error reading file: {}", err);
                        std::process::exit(1);
                    });
                    let mut modules_info = self.compiler.modules_info.borrow_mut();
                    modules_info
                        .modules_loaded
                        .insert(std::path::Path::new(&filename).to_path_buf());
                    modules_info.files.push(filename.to_string());
                }
                script
            }
        };
    }

    pub fn parse_script(&self) -> grammar::Program {
        grammar::parser_start(&self.script).unwrap_or_else(|err| {
            errors::print_error(
                &self.script,
                err.line,
                err.column,
                err.line,
                err.column,
                "parser error:",
                &format!("expected {}", {
                    let expected: Vec<String> =
                        err.expected.iter().map(|x| x.to_string()).collect();
                    expected.join(", ")
                }),
            );
            std::process::exit(1);
        })
    }

    pub fn emit_bytecode(&mut self, prog: grammar::Program) {
        for stmt in prog {
            if let Err(e) = stmt.emit(&mut self.compiler) {
                eprintln!("{:?}", e);
                return;
            }
        }
        self.compiler.cpushop(VmOpcode::Halt);
    }

    pub fn execute(&mut self) {
        self.compiler
            .modules_info
            .borrow_mut()
            .sources
            .push(self.script.clone());
        let mut vm = self.compiler.get_vm();
        hanayo::init(&mut vm);
        vm.gc_enable();
        vm.execute();
        errors::handle_error(&vm, &self.compiler);
    }

    pub fn run(&mut self) {
        self.load_script();
        let prog = self.parse_script();

        if self.flag.print_ast {
            println!("{:?}", prog);
            return;
        }

        self.emit_bytecode(prog);

        if self.flag.dump_bytecode {
            io::stdout().write_all(&[72, 97, 114, 117, 47, 47]).unwrap();
            io::stdout()
                .write_all(self.compiler.code_as_bytes())
                .unwrap();
            return;
        }

        self.execute();
    }
}

pub fn run_script(filepath: String, flags: ParserFlag) {
    let mut executor = ScriptExecutor::new(ExecutionKind::File(filepath), flags);
    executor.run();
}

pub fn run_cmd(cmd: String, flag: ParserFlag) {
    let mut executor = ScriptExecutor::new(ExecutionKind::Command(cmd), flag);
    executor.run();
}

pub fn run_repl(flag: ParserFlag) {
    repl::run_repl(flag);
}
