use crate::{
    ast, compiler,
    vmbindings::{vm::Vm, vmerror::VmError},
};
use ansi_term::Color as ac;

pub(crate) fn handle_error(vm: &Vm, c: &compiler::Compiler) -> bool {
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

pub(crate) fn print_error(
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
