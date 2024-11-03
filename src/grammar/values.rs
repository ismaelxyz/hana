use super::expressions::expr;
use super::statements::{func_statement, function_arguments, record_body_stmt};
use super::token::{
    eos, float_literal, id_chars, identifier, int_literal, skip_white, string_literal, white,
};
use super::{slice_eq, ParseState, RuleResult, RuleResult::*};
use crate::{ast, boxed};

fn value(input: &str, state: &mut ParseState, ps: usize) -> RuleResult<Box<dyn ast::Ast>> {
    // Find FLoat.
    if let Matched(pe, v) = float_literal(input, state, ps) {
        return Matched(pe, boxed!(FloatLiteral, ps, pe, val: v));
    }

    // Find Integer, String, identifier... (a value) if float not found.
    if let Matched(pe, v) = int_literal(input, state, ps) {
        return Matched(pe, boxed!(IntLiteral, ps, pe, val: v));
    }

    if let Matched(pe, v) = string_literal(input, state, ps) {
        return Matched(pe, boxed!(StrLiteral, ps, pe, val: v));
    }

    if let Matched(pe, v) = identifier(input, state, ps) {
        return Matched(pe, boxed!(Identifier, ps, pe, val: v));
    }

    match array_expr(input, state, ps) {
        Matched(pos, value) => Matched(pos, value),
        Failed => match record_expr(input, state, ps) {
            Matched(pos, value) => Matched(pos, value),
            Failed => match function_expr(input, state, ps) {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.suppress_fail += 1;
                    let res = {
                        match slice_eq(input, state, ps, "(") {
                            Matched(pos, _) => {
                                let seq_res = skip_white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = expr(input, state, pos);
                                        match seq_res {
                                            Matched(pos, e) => {
                                                let seq_res = skip_white(input, state, pos);
                                                match seq_res {
                                                    Matched(pos, _) => {
                                                        let seq_res =
                                                            slice_eq(input, state, pos, ")");
                                                        match seq_res {
                                                            Matched(pos, _) => Matched(pos, e),
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
                    state.suppress_fail -= 1;
                    res
                }
            },
        },
    }
}

fn array_expr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        let seq_res = Matched(pos, pos);
        match seq_res {
            Matched(pos, ps) => {
                let seq_res = {
                    state.suppress_fail += 1;
                    let res = {
                        let seq_res = slice_eq(input, state, pos, "[");
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = skip_white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => slice_eq(input, state, pos, "]"),
                                    Failed => Failed,
                                }
                            }
                            Failed => Failed,
                        }
                    };
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = Matched(pos, pos);
                        match seq_res {
                            Matched(pos, pe) => {
                                Matched(pos, boxed!(ArrayExpr, ps, pe, exprs: vec![]))
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
                            let res = slice_eq(input, state, pos, "[");
                            state.suppress_fail -= 1;
                            res
                        };
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = skip_white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = expr(input, state, pos);
                                        match seq_res {
                                            Matched(pos, fexpr) => {
                                                let seq_res = {
                                                    let mut repeat_pos = pos;
                                                    let mut repeat_value = vec![];
                                                    loop {
                                                        let pos = repeat_pos;
                                                        let step_res = {
                                                            let seq_res =
                                                                skip_white(input, state, pos);
                                                            match seq_res {
                                                                Matched(pos, _) => {
                                                                    let seq_res = slice_eq(
                                                                        input, state, pos, ",",
                                                                    );
                                                                    match seq_res {
                                                                        Matched(pos, _) => {
                                                                            let seq_res =
                                                                                skip_white(
                                                                                    input, state,
                                                                                    pos,
                                                                                );
                                                                            match seq_res {
                                                                                Matched(
                                                                                    pos,
                                                                                    _,
                                                                                ) => {
                                                                                    let seq_res =
                                                                                        expr(
                                                                                            input,
                                                                                            state,
                                                                                            pos,
                                                                                        );
                                                                                    match seq_res { Matched ( pos , e ) => { Matched (pos , e) } Failed => Failed , }
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
                                                    Matched(pos, lexpr) => {
                                                        let seq_res =
                                                            skip_white(input, state, pos);
                                                        match seq_res {
                                                            Matched(pos, _) => {
                                                                let seq_res = slice_eq(
                                                                    input, state, pos, "]",
                                                                );
                                                                match seq_res {
                                                                    Matched(pos, _) => {
                                                                        let seq_res =
                                                                            Matched(pos, pos);
                                                                        match seq_res {
                                                                            Matched(pos, pe) => {
                                                                                Matched(pos, {
                                                                                    let mut exprs =
                                                                                        vec![fexpr];
                                                                                    for expr in
                                                                                        lexpr
                                                                                    {
                                                                                        exprs
                                                                                            .push(
                                                                                            expr,
                                                                                        );
                                                                                    }
                                                                                    boxed!(
                                                                                        ArrayExpr,
                                                                                        ps,
                                                                                        pe,
                                                                                        exprs:
                                                                                            exprs
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
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    state.mark_failure(pos, "array literal");
                    Failed
                }
            }
        }
    }
}

pub(super) fn unary_expr(
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
                    let res = {
                        let str_start = pos;
                        match {
                            let choice_res = {
                                let seq_res = slice_eq(input, state, pos, "not");
                                match seq_res {
                                    Matched(pos, _) => {
                                        state.suppress_fail += 1;
                                        let __assert_res = match id_chars(input, state, pos) {
                                            Matched(pos, _) => Matched(pos, ()),
                                            Failed => Failed,
                                        };
                                        state.suppress_fail -= 1;
                                        match __assert_res {
                                            Failed => Matched(pos, ()),
                                            Matched(..) => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            match choice_res {
                                Matched(pos, value) => Matched(pos, value),
                                Failed => slice_eq(input, state, pos, "-"),
                            }
                        } {
                            Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                            Failed => Failed,
                        }
                    };
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, op) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = value(input, state, pos);
                                match seq_res {
                                    Matched(pos, val) => {
                                        let seq_res = Matched(pos, pos);
                                        match seq_res {
                                            Matched(pos, pe) => Matched(pos, {
                                                boxed!(UnaryExpr, ps, pe,
            op: match op {
                "not" => ast::UnaryOp::Not,
                "-" => ast::UnaryOp::Neg,
                &_ => unreachable!()
            },
            val: val)
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
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => value(input, state, pos),
    }
}

fn record_expr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let seq_res = Matched(pos, pos);
    match seq_res {
        Matched(pos, ps) => {
            let seq_res = slice_eq(input, state, pos, "record");
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
                                            let seq_res = slice_eq(input, state, pos, "end");
                                            match seq_res {
                                                Matched(pos, _) => {
                                                    let seq_res = Matched(pos, pos);
                                                    match seq_res {
                                                        Matched(pos, pe) => Matched(pos, {
                                                            boxed!(
                                                                RecordDefinition,
                                                                ps,
                                                                pe,
                                                                id: None,
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
}

fn function_expr(
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
                    let res = slice_eq(input, state, pos, "fn");
                    state.suppress_fail -= 1;
                    res
                };
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = white(input, state, pos);
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = function_arguments(input, state, pos);
                                match seq_res {
                                    Matched(pos, args) => {
                                        let seq_res = skip_white(input, state, pos);
                                        match seq_res {
                                            Matched(pos, _) => {
                                                let seq_res = {
                                                    let mut repeat_pos = pos;
                                                    let mut repeat_value = vec![];
                                                    loop {
                                                        let pos = repeat_pos;
                                                        let step_res =
                                                            func_statement(input, state, pos);
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
                                                        let seq_res =
                                                            skip_white(input, state, pos);
                                                        match seq_res {
                                                            Matched(pos, _) => {
                                                                let seq_res = slice_eq(
                                                                    input, state, pos, "end",
                                                                );
                                                                match seq_res {
                                                                    Matched(pos, _) => {
                                                                        let seq_res =
                                                                            Matched(pos, pos);
                                                                        match seq_res {
                                                                            Matched(pos, pe) => {
                                                                                Matched(pos, {
                                                                                    boxed!(FunctionDefinition, ps, pe,
            id: None,
            args: args,
            stmt: boxed!(BlockStatement, ps, pe, stmts: s))
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
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "anonymous function");
            Failed
        }
    }
}
