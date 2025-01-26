use clap::Parser;
use super::{ExecutionTarget, ParserFlag};

#[derive(Parser)]
#[clap(
    name = "hana",
    version = "0.29.9",
    author = "Haru Developers",
    about = "Interpreter Implemententation for the Hana Programming Language"
)]
pub struct CliArgs {
    #[arg(
        short,
        long,
        help = "execute program passed in as string",
        value_name = "INSTRUCTION"
    )]
    pub cmd: Option<String>,

    #[arg(
        short,
        long,
        help = "view the low-level instructions that the virtual machine will execute (only works in interpreter mode)"
    )]
    pub dump_bytecode: bool,

    #[arg(short, long, help = "runs file as bytecode")]
    pub bytecode: bool,

    #[arg(short, long, help = "prints ast and without run")]
    pub print_ast: bool,

    #[arg(help = "The name of the file to compile")]
    pub filename: Option<String>,
}


pub fn parse() -> (ExecutionTarget, ParserFlag) {
    let cli_args = CliArgs::parse();

    let flags = ParserFlag {
        dump_bytecode: cli_args.dump_bytecode,
        print_ast: cli_args.print_ast
    };

    let target = if let Some(instructions) = cli_args.cmd {
        ExecutionTarget::Cmd(instructions)
    } else if  let Some(filename) = cli_args.filename {
        ExecutionTarget::File(filename)
    } else {
        ExecutionTarget::Repl
    };

    (target, flags)
}
