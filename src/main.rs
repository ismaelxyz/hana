#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "jemalloc")] {
        extern crate jemallocator;
        #[global_allocator]
        static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
    }
}

use std::io::{self, Read, Write};
#[macro_use]
extern crate decorator;
extern crate ansi_term;
use ansi_term::Color as ac;

mod compiler;
mod grammar;
mod execution;
#[macro_use]
mod ast;
mod consts;
mod hanayo;
mod vmbindings;

use execution::{ParserFlag, ProcessArg};
use vmbindings::{
    vm::{Vm, VmOpcode},
    vmerror::VmError,
};

fn print_error(
    source: &str,
    lineno: usize,
    col: usize,
    _lineno_end: usize,
    col_end: usize,
    etype: &str,
    message: &str,
) {
    let line = source.split("\n").nth(lineno - 1).unwrap();
    let lineno_info = format!("{} | ", lineno);
    let lineno_info_len = lineno_info.len();
    eprintln!(
        "
{}{}
{}

{} {}",
        ac::Blue.bold().paint(lineno_info),
        line,
        ac::Blue.bold().paint(
            " ".repeat(lineno_info_len + col - 1)
                + &"^".repeat(if col_end > col { col_end - col } else { 1 })
        ),
        ac::Red.bold().paint(etype.to_string()),
        message
    );
}



fn process(arg: ProcessArg, flag: ParserFlag) {
    let mut c = compiler::Compiler::new(true);
    let s: String = match arg {
        ProcessArg::Command(cmd) => {
            c.modules_info
                .borrow_mut()
                .files
                .push("[cmdline]".to_string());
            cmd.to_string()
        }
        ProcessArg::File("-") => {
            let mut s: String = String::new();
            io::stdin().read_to_string(&mut s).unwrap_or_else(|err| {
                println!("error reading from stdin: {}", err);
                std::process::exit(1);
            });
            c.modules_info
                .borrow_mut()
                .files
                .push("[stdin]".to_string());
            s
        }
        ProcessArg::File(filename) => {
            let mut file = std::fs::File::open(filename).unwrap_or_else(|err| {
                println!("error opening file: {}", err);
                std::process::exit(1);
            });
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap_or_else(|err| {
                println!("error reading file: {}", err);
                std::process::exit(1);
            });
            let mut modules_info = c.modules_info.borrow_mut();
            modules_info
                .modules_loaded
                .insert(std::path::Path::new(&filename).to_path_buf());
            modules_info.files.push(filename.to_string());
            s
        }
    };
    let prog = grammar::parser_start(&s).unwrap_or_else(|err| {
        print_error(
            &s,
            err.line,
            err.column,
            err.line,
            err.column,
            "parser error:",
            &format!("expected {}", {
                let expected: Vec<String> = err.expected.iter().map(|x| x.to_string()).collect();
                expected.join(", ")
            }),
        );
        std::process::exit(1);
    });

    // dump ast if asked
    if flag.print_ast {
        println!("{:?}", prog);
        return;
    }

    // emit bytecode
    for stmt in prog {
        if let Err(e) = stmt.emit(&mut c) {
            // TODO: better error message
            eprintln!("{:?}", e);
            return;
        }
    }
    c.cpushop(VmOpcode::Halt);

    // dump bytecode if asked
    if flag.dump_bytecode {
        // 72, 97, 114, 117, 47, 47: Magic Mark
        io::stdout().write_all(&[72, 97, 114, 117, 47, 47]).unwrap();
        io::stdout().write_all(c.code_as_bytes()).unwrap();
        return;
    }

    // execute!
    c.modules_info.borrow_mut().sources.push(s);
    let mut vm = c.get_vm();
    hanayo::init(&mut vm);
    vm.gc_enable();
    vm.execute();
    handle_error(&vm, &c);
}

fn handle_error(vm: &Vm, c: &compiler::Compiler) -> bool {
    if vm.error != VmError::ERROR_NO_ERROR {
        if let Some(smap) = c.lookup_smap(vm.ip() as usize) {
            let src: &String = &c.modules_info.borrow().sources[smap.fileno];
            let (line, col) = ast::pos_to_line(src, smap.file.0);
            let (line_end, col_end) = ast::pos_to_line(src, smap.file.1);
            let message = format!(
                "{} at {}:{}:{}",
                vm.error,
                c.modules_info.borrow().files[smap.fileno],
                line,
                col
            );
            print_error(
                src,
                line,
                col,
                line_end,
                col_end,
                "interpreter error:",
                &message,
            );
        } else {
            println!("interpreter error: {}", vm.error);
            return true;
        }
        if let Some(hint) = unsafe { vm.error.hint(vm) } {
            eprintln!("{} {}", ac::Red.bold().paint("hint:"), hint);
        }
        let envs = vm.localenv_to_vec();
        if !envs.is_empty() {
            eprintln!("{}", ac::Red.bold().paint("backtrace:"));
            for env in envs {
                let ip = env.retip as usize;
                if let Some(smap) = c.lookup_smap(ip) {
                    let modules_info = c.modules_info.borrow();
                    let src = &modules_info.sources[smap.fileno];
                    let (line, col) = ast::pos_to_line(src, smap.file.0);
                    eprintln!(
                        " from {}{}:{}:{}",
                        if let Some(sym) = modules_info.symbol.get(&ip) {
                            sym.clone() + "@"
                        } else {
                            "".to_string()
                        },
                        modules_info.files[smap.fileno],
                        line,
                        col
                    );
                } else {
                    eprintln!(" from bytecode index {}", ip);
                }
            }
        }
        true
    } else {
        false
    }
}




fn main() {
    use execution::{ExecutionTarget, cli, repl};
    let (target, flags) = cli::parse();


    match target {
        ExecutionTarget::Cmd(instructions) => process(ProcessArg::Command(&instructions), flags),
        ExecutionTarget::File(filename) => process(ProcessArg::File(&filename), flags),
        ExecutionTarget::Repl => {
            println!("{}", consts::VERSION);
            repl::run_repl(flags)
        }
    }
}
