use std::rc::Rc;

use super::{
    errors::{handle_error, print_error},
    ParserFlag,
};

use crate::harumachine::vm::VmOpcode;
use crate::harumachine::vmerror::VmError;
use crate::{ast, grammar};
use crate::{
    compiler, hanayo,
    harumachine::vm::{ execute_vm, initialize_vm},
};
use rustyline::{error::ReadlineError, history::DefaultHistory, Editor};

// repl
pub(crate) fn run_repl(flag: ParserFlag) {
    let mut rl = Editor::<(), DefaultHistory>::new().unwrap();
    let mut c = compiler::Compiler::new(false);
    {
        let mut modules_info = c.modules_info.borrow_mut();
        modules_info.files.push("[repl]".to_string());
        modules_info.sources.push(String::new());
    }
    let vm = initialize_vm(Vec::new(), Some(c.modules_info.clone()), None);
    hanayo::init(Rc::clone(&vm));

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(s) => {
                rl.add_history_entry(s.as_str()).unwrap();
                c.modules_info.borrow_mut().sources[0] = s.clone();
                match grammar::parser_start(&s) {
                    Ok(mut prog) => {
                        if flag.print_ast {
                            println!("{:?}", prog);
                            continue;
                        }
                        let gencode =
                            |c: &mut compiler::Compiler| -> Result<bool, ast::CodeGenError> {
                                if prog.last().is_some() {
                                    let stmt = prog.pop().unwrap();
                                    for stmt in prog {
                                        stmt.emit(c)?;
                                    }
                                    if let Some(expr_stmt) =
                                        stmt.as_any().downcast_ref::<ast::ExprStatement>()
                                    {
                                        expr_stmt.expr.emit(c)?;
                                        return Ok(true);
                                    } else {
                                        stmt.emit(c)?;
                                    }
                                } else {
                                    for stmt in prog {
                                        stmt.emit(c)?;
                                    }
                                }
                                Ok(false)
                            };
                        // setup

                        let pop_print: bool; // false
                        if (*vm).borrow().code.is_empty() {
                            match gencode(&mut c) {
                                Ok(pop_print_) => {
                                    pop_print = pop_print_;
                                    c.cpushop(VmOpcode::Halt);
                                    vm.borrow_mut().code = c.take_code();
                                    execute_vm(Rc::clone(&vm));
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            }
                        } else {
                            vm.borrow_mut().error = VmError::ERROR_NO_ERROR;
                            let len = (*vm).borrow().code.len() as u32;
                            c.receive_code((*vm).borrow().code.clone());
                            match gencode(&mut c) {
                                Ok(pop_print_) => {
                                    pop_print = pop_print_;
                                    if c.clen() as u32 == len {
                                        continue;
                                    }
                                    c.cpushop(VmOpcode::Halt);
                                    vm.borrow_mut().code = c.take_code();
                                    vm.borrow_mut().jmp(len);
                                    execute_vm(Rc::clone(&vm));
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            }
                        }
                        if !handle_error(Rc::clone(&vm), &c) && pop_print {
                            println!("=> {:?}", vm.borrow_mut().stack.pop().unwrap());
                        }
                    }
                    Err(err) => {
                        print_error(
                            &s,
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
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
