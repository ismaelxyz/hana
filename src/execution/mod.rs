pub (crate) mod cli;
pub (crate) mod repl;

pub enum ExecutionTarget {
    Cmd(String),
    File(String),
    Repl,
}

pub struct ParserFlag {
    pub dump_bytecode: bool,
    pub print_ast: bool,
}


// command/file
pub enum ProcessArg<'a> {
    Command(&'a str),
    File(&'a str),
}

// fn init_execution() {

// }