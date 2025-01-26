use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "hana",
    version = "0.29.9",
    author = "Haru Developers",
    about = "Interpreter Implemententation for the Hana Programming Language"
)]
pub(crate) struct CliArgs {
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

impl CliArgs {
    pub(crate) fn parse_args() -> CliArgs {
        CliArgs::parse()
    }
}
