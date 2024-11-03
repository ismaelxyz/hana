use super::expressions::expr;
use super::token::{eos, identifier, skip_white, string_literal, white, word};
use super::{slice_eq, ParseState};
use super::{RuleResult, RuleResult::*};
use crate::{ast, boxed};

fn statement_program_no_eos(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = block_stmt(input, state, pos);
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = if_stmt(input, state, pos);
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    let choice_res = while_stmt(input, state, pos);
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => {
                            let choice_res = for_in_stmt(input, state, pos);
                            match choice_res {
                                Matched(pos, value) => Matched(pos, value),
                                Failed => {
                                    let choice_res = function_stmt(input, state, pos);
                                    match choice_res {
                                        Matched(pos, value) => Matched(pos, value),
                                        Failed => {
                                            let choice_res = record_stmt(input, state, pos);
                                            match choice_res {
                                                Matched(pos, value) => Matched(pos, value),
                                                Failed => {
                                                    let choice_res = try_stmt(input, state, pos);
                                                    match choice_res {
                                                        Matched(pos, value) => Matched(pos, value),
                                                        Failed => {
                                                            let choice_res =
                                                                raise_stmt(input, state, pos);
                                                            match choice_res {
                                                                Matched(pos, value) => {
                                                                    Matched(pos, value)
                                                                }
                                                                Failed => {
                                                                    let choice_res = use_stmt(
                                                                        input, state, pos,
                                                                    );
                                                                    match choice_res {
                                                                        Matched(pos, value) => {
                                                                            Matched(pos, value)
                                                                        }
                                                                        Failed => expr_stmt(
                                                                            input, state, pos,
                                                                        ),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn statement_program(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = skip_white(input, state, pos);
        match seq_res {
            Matched(pos, _) => {
                let seq_res = statement_program_no_eos(input, state, pos);
                match seq_res {
                    Matched(pos, s) => {
                        let seq_res = skip_white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => Matched(pos, s),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "statement");
            Failed
        }
    }
}

fn statement_no_eos(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = statement_program_no_eos(input, state, pos);
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = return_stmt(input, state, pos);
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    let choice_res = continue_stmt(input, state, pos);
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => break_stmt(input, state, pos),
                    }
                }
            }
        }
    }
}

fn statement(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = skip_white(input, state, pos);
        match seq_res {
            Matched(pos, _) => {
                let seq_res = statement_no_eos(input, state, pos);
                match seq_res {
                    Matched(pos, s) => {
                        let seq_res = skip_white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => Matched(pos, s),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "statement");
            Failed
        }
    }
}

pub(super) fn func_statement(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = skip_white(input, state, pos);
        match seq_res {
            Matched(pos, _) => {
                let seq_res = statement_program_no_eos(input, state, pos);
                match seq_res {
                    Matched(pos, s) => {
                        let seq_res = skip_white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => Matched(pos, s),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                let seq_res = skip_white(input, state, pos);
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = return_stmt(input, state, pos);
                        match seq_res {
                            Matched(pos, s) => {
                                let seq_res = skip_white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => Matched(pos, s),
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.mark_failure(pos, "statement");
                    Failed
                }
            }
        }
    }
}

fn block_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "begin");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = eos(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = {
                                    let mut repeat_pos = pos;
                                    let mut repeat_value = vec![];
                                    loop {
                                        let pos = repeat_pos;
                                        let step_res = statement(input, state, pos);
                                        match step_res {
                                            Matched(newpos, value) => {
                                                repeat_pos = newpos;
                                                repeat_value.push(value);
                                            }
                                            Failed => {
                                                break;
                                            }
                                        }
                                    }
                                    Matched(repeat_pos, repeat_value)
                                };
                                match seq_res {
                                    Matched(pos, s) => {
                                        let seq_res = skip_white(input, state, pos);
                                        match seq_res {
                                            Matched(pos, _) => {
                                                let seq_res = slice_eq(input, state, pos, "end");
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res = Matched(pos, pos);
                                                        match seq_res {
                                                            Matched(pos, pe) => Matched(pos, {
                                                                boxed!(
                                                                    BlockStatement,
                                                                    ps,
                                                                    pe,
                                                                    stmts: s
                                                                )
                                                            }),
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                let seq_res = Matched(pos, pos);
                match seq_res {
                    Matched(pos, ps) => {
                        let seq_res = {
                            state.suppress_fail += 1;
                            let res = slice_eq(input, state, pos, "begin");
                            state.suppress_fail -= 1;
                            res
                        };
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = eos(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = skip_white(input, state, pos);
                                        match seq_res {
                                            Matched(pos, _) => {
                                                let seq_res = slice_eq(input, state, pos, "end");
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res = Matched(pos, pos);
                                                        match seq_res {
                                                            Matched(pos, pe) => Matched(pos, {
                                                                boxed!(
                                                                    BlockStatement,
                                                                    ps,
                                                                    pe,
                                                                    stmts: Vec::new()
                                                                )
                                                            }),
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.mark_failure(pos, "block statement");
                    Failed
                }
            }
        }
    }
}

fn then_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            let seq_res = slice_eq(input, state, pos, "then");
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = statement(input, state, pos);
                    match seq_res {
                        Matched(pos, s) => Matched(pos, s),
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        };
        state.suppress_fail -= 1;
        res
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                state.suppress_fail += 1;
                let res = block_stmt(input, state, pos);
                state.suppress_fail -= 1;
                res
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.mark_failure(pos, "block or then <stmt>");
                    Failed
                }
            }
        }
    }
}

fn if_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "if");
                state.suppress_fail -= 1;
                res
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = white(input, state, pos);
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = expr(input, state, pos);
                            match seq_res {
                                Matched(pos, e) => {
                                    let seq_res = white(input, state, pos);
                                    match seq_res {
                                        Matched(pos, _) => {
                                            let seq_res = then_stmt(input, state, pos);
                                            match seq_res {
                                                Matched(pos, s) => {
                                                    let seq_res = match {
                                                        let seq_res =
                                                            skip_white(input, state, pos);
                                                        match seq_res {
                                                            Matched(pos, _) => {
                                                                let seq_res = slice_eq(
                                                                    input, state, pos, "else",
                                                                );
                                                                match seq_res {
                                                                    Matched(pos, _) => {
                                                                        let seq_res = white(
                                                                            input, state, pos,
                                                                        );
                                                                        match seq_res {
                                                                            Matched(pos, _) => {
                                                                                let seq_res =
                                                                                    statement(
                                                                                        input,
                                                                                        state,
                                                                                        pos,
                                                                                    );
                                                                                match seq_res {
                                                                                    Matched(
                                                                                        pos,
                                                                                        s,
                                                                                    ) => Matched(
                                                                                        pos, s,
                                                                                    ),
                                                                                    Failed => {
                                                                                        Failed
                                                                                    }
                                                                                }
                                                                            }
                                                                            Failed => Failed,
                                                                        }
                                                                    }
                                                                    Failed => Failed,
                                                                }
                                                            }
                                                            Failed => Failed,
                                                        }
                                                    } {
                                                        Matched(newpos, value) => {
                                                            Matched(newpos, Some(value))
                                                        }
                                                        Failed => Matched(pos, None),
                                                    };
                                                    match seq_res {
                                                        Matched(pos, a) => {
                                                            let seq_res = Matched(pos, pos);
                                                            match seq_res {
                                                                Matched(pos, pe) => {
                                                                    Matched(pos, {
                                                                        boxed!(
                                                                            IfStatement,
                                                                            ps,
                                                                            pe,
                                                                            expr: e,
                                                                            then: s,
                                                                            alt: a
                                                                        )
                                                                    })
                                                                }
                                                                Failed => Failed,
                                                            }
                                                        }
                                                        Failed => Failed,
                                                    }
                                                }
                                                Failed => Failed,
                                            }
                                        }
                                        Failed => Failed,
                                    }
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn while_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "while");
                state.suppress_fail -= 1;
                res
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = white(input, state, pos);
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = expr(input, state, pos);
                            match seq_res {
                                Matched(pos, e) => {
                                    let seq_res = white(input, state, pos);
                                    match seq_res {
                                        Matched(pos, _) => {
                                            let seq_res = then_stmt(input, state, pos);
                                            match seq_res {
                                                Matched(pos, s) => {
                                                    let seq_res = Matched(pos, pos);
                                                    match seq_res {
                                                        Matched(pos, pe) => Matched(pos, {
                                                            boxed!(
                                                                WhileStatement,
                                                                ps,
                                                                pe,
                                                                expr: e,
                                                                then: s
                                                            )
                                                        }),
                                                        Failed => Failed,
                                                    }
                                                }
                                                Failed => Failed,
                                            }
                                        }
                                        Failed => Failed,
                                    }
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn for_in_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "for");
                state.suppress_fail -= 1;
                res
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = white(input, state, pos);
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = identifier(input, state, pos);
                            match seq_res {
                                Matched(pos, id) => {
                                    let seq_res = white(input, state, pos);
                                    match seq_res {
                                        Matched(pos, _) => {
                                            let seq_res = slice_eq(input, state, pos, "in");
                                            match seq_res {
                                                Matched(pos, _) => {
                                                    let seq_res = white(input, state, pos);
                                                    match seq_res {
                                                        Matched(pos, _) => {
                                                            let seq_res = expr(input, state, pos);
                                                            match seq_res {
                                                                Matched(pos, expr) => {
                                                                    let seq_res =
                                                                        white(input, state, pos);
                                                                    match seq_res {
                                                                        Matched(pos, _) => {
                                                                            let seq_res =
                                                                                then_stmt(
                                                                                    input, state,
                                                                                    pos,
                                                                                );
                                                                            match seq_res {
                                                                                Matched(
                                                                                    pos,
                                                                                    s,
                                                                                ) => {
                                                                                    let seq_res =
                                                                                        Matched(
                                                                                            pos,
                                                                                            pos,
                                                                                        );
                                                                                    match seq_res { Matched ( pos , pe ) => {
                                                                                        Matched(pos , {
        boxed!(ForInStatement, ps, pe,
            id: id,
            expr: expr,
            stmt: s)
     } ) } Failed => Failed , }
                                                                                }
                                                                                Failed => Failed,
                                                                            }
                                                                        }
                                                                        Failed => Failed,
                                                                    }
                                                                }
                                                                Failed => Failed,
                                                            }
                                                        }
                                                        Failed => Failed,
                                                    }
                                                }
                                                Failed => Failed,
                                            }
                                        }
                                        Failed => Failed,
                                    }
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn continue_stmt(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "continue");
                state.suppress_fail -= 1;
                res
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = Matched(pos, pos);
                    match seq_res {
                        Matched(pos, pe) => Matched(pos, boxed!(ContinueStatement, ps, pe,)),
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn break_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "break");
                state.suppress_fail -= 1;
                res
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = Matched(pos, pos);
                    match seq_res {
                        Matched(pos, pe) => Matched(pos, boxed!(BreakStatement, ps, pe,)),
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn try_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    #![allow(non_snake_case, unused)]
    {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "try");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = eos(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = {
                                    let mut repeat_pos = pos;
                                    let mut repeat_value = vec![];
                                    loop {
                                        let pos = repeat_pos;
                                        let step_res = statement(input, state, pos);
                                        match step_res {
                                            Matched(newpos, value) => {
                                                repeat_pos = newpos;
                                                repeat_value.push(value);
                                            }
                                            Failed => {
                                                break;
                                            }
                                        }
                                    }
                                    Matched(repeat_pos, repeat_value)
                                };
                                match seq_res {
                                    Matched(pos, stmts) => {
                                        let seq_res = {
                                            let mut repeat_pos = pos;
                                            let mut repeat_value = vec![];
                                            loop {
                                                let pos = repeat_pos;
                                                let step_res = case_stmt(input, state, pos);
                                                match step_res {
                                                    Matched(newpos, value) => {
                                                        repeat_pos = newpos;
                                                        repeat_value.push(value);
                                                    }
                                                    Failed => {
                                                        break;
                                                    }
                                                }
                                            }
                                            Matched(repeat_pos, repeat_value)
                                        };
                                        match seq_res {
                                            Matched(pos, cases) => {
                                                let seq_res = slice_eq(input, state, pos, "end");
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res = Matched(pos, pos);
                                                        match seq_res {
                                                            Matched(pos, pe) => Matched(pos, {
                                                                boxed!(
                                                                    TryStatement,
                                                                    ps,
                                                                    pe,
                                                                    stmts: stmts,
                                                                    cases: cases
                                                                )
                                                            }),
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    }
}

fn case_stmt(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<ast::CaseStatement>> {
    #![allow(non_snake_case, unused)]
    {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "case");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = expr(input, state, pos);
                                match seq_res {
                                    Matched(pos, etype) => {
                                        let seq_res = match {
                                            let seq_res = white(input, state, pos);
                                            match seq_res {
                                                Matched(pos, _) => {
                                                    let seq_res =
                                                        slice_eq(input, state, pos, "as");
                                                    match seq_res {
                                                        Matched(pos, _) => {
                                                            let seq_res = white(input, state, pos);
                                                            match seq_res {
                                                                Matched(pos, _) => {
                                                                    let seq_res =
                                                                        expr(input, state, pos);
                                                                    match seq_res {
                                                                        Matched(pos, t) => {
                                                                            Matched(pos, { t })
                                                                        }
                                                                        Failed => Failed,
                                                                    }
                                                                }
                                                                Failed => Failed,
                                                            }
                                                        }
                                                        Failed => Failed,
                                                    }
                                                }
                                                Failed => Failed,
                                            }
                                        } {
                                            Matched(newpos, value) => Matched(newpos, Some(value)),
                                            Failed => Matched(pos, None),
                                        };
                                        match seq_res {
                                            Matched(pos, id) => {
                                                let seq_res = eos(input, state, pos);
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res = {
                                                            let mut repeat_pos = pos;
                                                            let mut repeat_value = vec![];
                                                            loop {
                                                                let pos = repeat_pos;
                                                                let step_res =
                                                                    statement(input, state, pos);
                                                                match step_res {
                                                                    Matched(newpos, value) => {
                                                                        repeat_pos = newpos;
                                                                        repeat_value.push(value);
                                                                    }
                                                                    Failed => {
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            Matched(repeat_pos, repeat_value)
                                                        };
                                                        match seq_res {
                                                            Matched(pos, stmts) => {
                                                                let seq_res = Matched(pos, pos);
                                                                match seq_res {
                                                                    Matched(pos, pe) => {
                                                                        Matched(pos, {
                                                                            boxed!(
                                                                                CaseStatement,
                                                                                ps,
                                                                                pe,
                                                                                etype: etype,
                                                                                id: id,
                                                                                stmts: stmts
                                                                            )
                                                                        })
                                                                    }
                                                                    Failed => Failed,
                                                                }
                                                            }
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    }
}

fn raise_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    #![allow(non_snake_case, unused)]
    {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "raise");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = expr(input, state, pos);
                                match seq_res {
                                    Matched(pos, expr) => {
                                        let seq_res = Matched(pos, pos);
                                        match seq_res {
                                            Matched(pos, pe) => Matched(pos, {
                                                boxed!(RaiseStatement, ps, pe, expr: expr)
                                            }),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    }
}

fn use_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    #![allow(non_snake_case, unused)]
    {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "use");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = string_literal(input, state, pos);
                                match seq_res {
                                    Matched(pos, path) => {
                                        let seq_res = Matched(pos, pos);
                                        match seq_res {
                                            Matched(pos, pe) => Matched(pos, {
                                                boxed!(UseStatement, ps, pe, path: path)
                                            }),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    }
}

pub(super) fn function_arguments(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Vec<String>> {
    let choice_res = {
        let seq_res = slice_eq(input, state, pos, "(");
        match seq_res {
            Matched(pos, _) => {
                let seq_res = skip_white(input, state, pos);
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = slice_eq(input, state, pos, ")");
                        match seq_res {
                            Matched(pos, _) => Matched(pos, Vec::new()),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                let seq_res = slice_eq(input, state, pos, "(");
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = skip_white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = identifier(input, state, pos);
                                match seq_res {
                                    Matched(pos, fid) => {
                                        let seq_res = {
                                            let mut repeat_pos = pos;
                                            let mut repeat_value = vec![];
                                            loop {
                                                let pos = repeat_pos;
                                                let step_res = {
                                                    let seq_res = skip_white(input, state, pos);
                                                    match seq_res {
                                                        Matched(pos, _) => {
                                                            let seq_res =
                                                                slice_eq(input, state, pos, ",");
                                                            match seq_res {
                                                                Matched(pos, _) => {
                                                                    let seq_res = skip_white(
                                                                        input, state, pos,
                                                                    );
                                                                    match seq_res {
                                                                        Matched(pos, _) => {
                                                                            let seq_res =
                                                                                identifier(
                                                                                    input, state,
                                                                                    pos,
                                                                                );
                                                                            match seq_res {
                                                                                Matched(
                                                                                    pos,
                                                                                    id,
                                                                                ) => {
                                                                                    Matched(pos, {
                                                                                        id
                                                                                    })
                                                                                }
                                                                                Failed => Failed,
                                                                            }
                                                                        }
                                                                        Failed => Failed,
                                                                    }
                                                                }
                                                                Failed => Failed,
                                                            }
                                                        }
                                                        Failed => Failed,
                                                    }
                                                };
                                                match step_res {
                                                    Matched(newpos, value) => {
                                                        repeat_pos = newpos;
                                                        repeat_value.push(value);
                                                    }
                                                    Failed => {
                                                        break;
                                                    }
                                                }
                                            }
                                            Matched(repeat_pos, repeat_value)
                                        };
                                        match seq_res {
                                            Matched(pos, lid) => {
                                                let seq_res = skip_white(input, state, pos);
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res =
                                                            slice_eq(input, state, pos, ")");
                                                        match seq_res {
                                                            Matched(pos, _) => Matched(pos, {
                                                                let mut v = vec![fid];
                                                                for id in lid {
                                                                    v.push(id.to_string());
                                                                }
                                                                v
                                                            }),
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.mark_failure(pos, "function arguments");
                    Failed
                }
            }
        }
    }
}

fn function_stmt(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = slice_eq(input, state, pos, "func");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = word(input, state, pos);
                                match seq_res {
                                    Matched(pos, id) => {
                                        let seq_res = white(input, state, pos);
                                        match seq_res {
                                            Matched(pos, _) => {
                                                let seq_res =
                                                    function_arguments(input, state, pos);
                                                match seq_res {
                                                    Matched(pos, args) => {
                                                        let seq_res =
                                                            skip_white(input, state, pos);
                                                        match seq_res {
                                                            Matched(pos, _) => {
                                                                let seq_res = {
                                                                    let mut repeat_pos = pos;
                                                                    let mut repeat_value = vec![];
                                                                    loop {
                                                                        let pos = repeat_pos;
                                                                        let step_res =
                                                                            func_statement(
                                                                                input, state, pos,
                                                                            );
                                                                        match step_res {
                                                                            Matched(
                                                                                newpos,
                                                                                value,
                                                                            ) => {
                                                                                repeat_pos =
                                                                                    newpos;
                                                                                repeat_value
                                                                                    .push(value);
                                                                            }
                                                                            Failed => {
                                                                                break;
                                                                            }
                                                                        }
                                                                    }
                                                                    Matched(
                                                                        repeat_pos,
                                                                        repeat_value,
                                                                    )
                                                                };
                                                                match seq_res {
                                                                    Matched(pos, s) => {
                                                                        let seq_res = skip_white(
                                                                            input, state, pos,
                                                                        );
                                                                        match seq_res {
                                                                            Matched(pos, _) => {
                                                                                let seq_res =
                                                                                    slice_eq(
                                                                                        input,
                                                                                        state,
                                                                                        pos,
                                                                                        "end",
                                                                                    );
                                                                                match seq_res {
                                                                                    Matched(
                                                                                        pos,
                                                                                        _,
                                                                                    ) => {
                                                                                        let seq_res = Matched (pos , pos);
                                                                                        match seq_res { Matched (pos, pe) => { Matched(pos , {
        Box::new(ast::FunctionStatement::new(ast::FunctionDefinition {
            _span: (ps, pe),
            id: Some(id),
            args,
            stmt: boxed!(BlockStatement, ps, pe, stmts: s)
        }, (ps, pe)))
     } ) } Failed => Failed , }
                                                                                    }
                                                                                    Failed => {
                                                                                        Failed
                                                                                    }
                                                                                }
                                                                            }
                                                                            Failed => Failed,
                                                                        }
                                                                    }
                                                                    Failed => Failed,
                                                                }
                                                            }
                                                            Failed => Failed,
                                                        }
                                                    }
                                                    Failed => Failed,
                                                }
                                            }
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "function");
            Failed
        }
    }
}

fn return_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let ps = pos;
    let seq_res = {
        state.suppress_fail += 1;
        let res = slice_eq(input, state, pos, "return");
        state.suppress_fail -= 1;
        res
    };
    match seq_res {
        Matched(pos, _) => {
            let seq_res = match {
                let seq_res = white(input, state, pos);
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = expr(input, state, pos);
                        match seq_res {
                            Matched(pos, e) => Matched(pos, e),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            } {
                Matched(newpos, value) => Matched(newpos, Some(value)),
                Failed => Matched(pos, None),
            };
            match seq_res {
                Matched(pos, e) => {
                    let seq_res = eos(input, state, pos);
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = Matched(pos, pos);
                            match seq_res {
                                Matched(pos, pe) => {
                                    Matched(pos, boxed!(ReturnStatement, ps, pe, expr: e))
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

pub(super) fn record_body_stmt(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    // TODO: Recurda la nota de start!
    let choice_res = {
        match skip_white(input, state, pos) {
            Matched(pos, _) => {
                let seq_res = {
                    match function_stmt(input, state, pos) {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => match record_stmt(input, state, pos) {
                            Matched(pos, value) => Matched(pos, value),
                            Failed => expr_stmt(input, state, pos),
                        },
                    }
                };
                match seq_res {
                    Matched(pos, s) => match skip_white(input, state, pos) {
                        Matched(pos, _) => Matched(pos, s),
                        Failed => Failed,
                    },
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "statement");
            Failed
        }
    }
}

fn record_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let ps = pos;
    let seq_res = {
        state.suppress_fail += 1;
        let res = slice_eq(input, state, pos, "record");
        state.suppress_fail -= 1;
        res
    };
    match seq_res {
        Matched(pos, _) => {
            let seq_res = white(input, state, pos);
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = identifier(input, state, pos);
                    match seq_res {
                        Matched(pos, id) => {
                            let seq_res = eos(input, state, pos);
                            match seq_res {
                                Matched(pos, _) => {
                                    let seq_res = {
                                        let mut repeat_pos = pos;
                                        let mut repeat_value = vec![];
                                        loop {
                                            let pos = repeat_pos;
                                            let step_res = record_body_stmt(input, state, pos);
                                            match step_res {
                                                Matched(newpos, value) => {
                                                    repeat_pos = newpos;
                                                    repeat_value.push(value);
                                                }
                                                Failed => {
                                                    break;
                                                }
                                            }
                                        }
                                        Matched(repeat_pos, repeat_value)
                                    };
                                    match seq_res {
                                        Matched(pos, s) => {
                                            let seq_res = white(input, state, pos);
                                            match seq_res {
                                                Matched(pos, _) => {
                                                    let seq_res =
                                                        slice_eq(input, state, pos, "end");
                                                    match seq_res {
                                                        Matched(pos, _) => {
                                                            let seq_res = eos(input, state, pos);
                                                            match seq_res {
                                                                Matched(pos, _) => {
                                                                    let seq_res =
                                                                        Matched(pos, pos);
                                                                    match seq_res {
                                                                        Matched(pos, pe) => {
                                                                            Matched(pos, {
                                                                                Box::new(ast::RecordStatement::new(ast::RecordDefinition {
            _span: (ps, pe),
            id: Some(id),
            stmts: s
        }, (ps, pe)))
                                                                            })
                                                                        }
                                                                        Failed => Failed,
                                                                    }
                                                                }
                                                                Failed => Failed,
                                                            }
                                                        }
                                                        Failed => Failed,
                                                    }
                                                }
                                                Failed => Failed,
                                            }
                                        }
                                        Failed => Failed,
                                    }
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn expr_stmt(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let ps = pos;
    match expr(input, state, pos) {
        Matched(pos, s) => match eos(input, state, pos) {
            Matched(pe, _) => Matched(pos, boxed!(ExprStatement, ps, pe, expr: s)),
            Failed => Failed,
        },
        Failed => Failed,
    }
}
