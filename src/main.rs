#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "jemalloc")] {
        extern crate jemallocator;
        #[global_allocator]
        static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
    }
}

#[macro_use]
extern crate decorator;
extern crate ansi_term;

mod compiler;
mod execution;
mod grammar;
#[macro_use]
mod ast;
mod cli;
mod consts;
mod hanayo;
mod harumachine;

use cli::CliArgs;
use execution::{run_cmd, run_repl, run_script, ParserFlag};

fn main() {
    let cli_args = CliArgs::parse_args();

    let flags = ParserFlag {
        dump_bytecode: cli_args.dump_bytecode,
        print_ast: cli_args.print_ast,
    };

    if let Some(instructions) = cli_args.cmd {
        run_cmd(instructions, flags);
    } else if let Some(filename) = cli_args.filename {
        run_script(filename, flags);
    } else {
        println!("{}", consts::VERSION);
        run_repl(flags);
    }
}
