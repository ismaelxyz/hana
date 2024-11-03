use super::token::{parse_s, skip_white, white, word};
use super::values::unary_expr;
use super::{slice_eq, ParseState, RuleResult, RuleResult::*};
use crate::{ast, boxed};

pub(super) fn expr(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    match assignmentexpr(input, state, pos) {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "expression");
            Failed
        }
    }
}

fn assignmentexpr(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        fn infix_parse(
            min_prec: i32,
            input: &str,
            state: &mut ParseState,
            pos: usize,
        ) -> RuleResult<Box<dyn ast::Ast>> {
            if let Matched(pos, mut infix_result) = condexpr(input, state, pos) {
                let mut repeat_pos = pos;
                loop {
                    let pos = repeat_pos;
                    if 0i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Assign
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "+=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Adds
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "-=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Subs
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "*=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Muls
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "/=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Divs
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "%=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Mods
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    break;
                }
                Matched(repeat_pos, infix_result)
            } else {
                Failed
            }
        }
        infix_parse(0, input, state, pos)
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => condexpr(input, state, pos),
    }
}

// 1 == 1 ? true : false
fn condexpr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            let ps = pos;
            match binexpr(input, state, pos) {
                Matched(pos, cond) => match white(input, state, pos) {
                    Matched(pos, _) => match slice_eq(input, state, pos, "?") {
                        Matched(pos, _) => match white(input, state, pos) {
                            Matched(pos, _) => match binexpr(input, state, pos) {
                                Matched(pos, then) => match white(input, state, pos) {
                                    Matched(pos, _) => match slice_eq(input, state, pos, ":") {
                                        Matched(pos, _) => match white(input, state, pos) {
                                            Matched(pos, _) => match binexpr(input, state, pos) {
                                                // Here Return!
                                                Matched(pe, alt) => Matched(pe, {
                                                    boxed!(
                                                        CondExpr,
                                                        ps,
                                                        pe,
                                                        cond: cond,
                                                        then: then,
                                                        alt: alt
                                                    )
                                                }),
                                                Failed => Failed,
                                            },
                                            Failed => Failed,
                                        },
                                        Failed => Failed,
                                    },
                                    Failed => Failed,
                                },
                                Failed => Failed,
                            },
                            Failed => Failed,
                        },
                        Failed => Failed,
                    },
                    Failed => Failed,
                },
                Failed => Failed,
            }
        };
        state.suppress_fail -= 1;
        res
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => binexpr(input, state, pos),
    }
}

fn binexpr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        fn infix_parse(
            min_prec: i32,
            input: &str,
            state: &mut ParseState,
            pos: usize,
        ) -> RuleResult<Box<dyn ast::Ast>> {
            if let Matched(pos, mut infix_result) = callexpr(input, state, pos) {
                let mut repeat_pos = pos;
                loop {
                    let pos = repeat_pos;
                    if 0i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "&");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::BitwiseAnd
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "|");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::BitwiseOr
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = parse_s(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "~");
                                        match seq_res {
                                            Matched(pos, _) => parse_s(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(1i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::BitwiseXor
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    if 1i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = parse_s(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "and");
                                        match seq_res {
                                            Matched(pos, _) => parse_s(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(2i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::And
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = parse_s(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "or");
                                        match seq_res {
                                            Matched(pos, _) => parse_s(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(2i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Or
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    if 2i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "==");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Eq
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "!=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Neq
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, ">");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Gt
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "<");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Lt
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, ">=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Geq
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "<=");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(3i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Leq
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    if 3i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = parse_s(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "of");
                                        match seq_res {
                                            Matched(pos, _) => parse_s(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(4i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Of
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    if 4i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "+");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(5i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Add
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "-");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(5i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Sub
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    if 5i32 >= min_prec {
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "*");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(6i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Mul
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = white(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "/");
                                        match seq_res {
                                            Matched(pos, _) => white(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(6i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Div
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                        if let Matched(pos, _) = {
                            state.suppress_fail += 1;
                            let res = {
                                let seq_res = parse_s(input, state, pos);
                                match seq_res {
                                    Matched(pos, _) => {
                                        let seq_res = slice_eq(input, state, pos, "%");
                                        match seq_res {
                                            Matched(pos, _) => parse_s(input, state, pos),
                                            Failed => Failed,
                                        }
                                    }
                                    Failed => Failed,
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            if let Matched(pos, y) = infix_parse(6i32, input, state, pos) {
                                let x = infix_result;
                                infix_result = {
                                    boxed!(
                                        BinExpr,
                                        x.span().0,
                                        y.span().1,
                                        left: x,
                                        right: y,
                                        op: ast::BinOp::Mod
                                    )
                                };
                                repeat_pos = pos;
                                continue;
                            }
                        }
                    }
                    break;
                }
                Matched(repeat_pos, infix_result)
            } else {
                Failed
            }
        }
        infix_parse(0, input, state, pos)
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => callexpr(input, state, pos),
    }
}

fn callexpr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            let ps = pos;
            match memexpr(input, state, pos) {
                Matched(pos, _left) => {
                    let seq_res = white(input, state, pos);
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = callexpr_args(input, state, pos);
                            match seq_res {
                                Matched(pos, args) => {
                                    let seq_res = {
                                        let mut repeat_pos = pos;
                                        let mut repeat_value = vec![];
                                        loop {
                                            let pos = repeat_pos;
                                            let step_res = callexpr_arm(input, state, pos);
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
                                        Matched(pos, _right) => {
                                            let seq_res = Matched(pos, pos);
                                            match seq_res {
                                                Matched(pos, pe) => Matched(pos, {
                                                    let mut left: Box<dyn ast::Ast> = boxed!(
                                                        CallExpr,
                                                        ps,
                                                        pe,
                                                        callee: _left,
                                                        args: args
                                                    );
                                                    for right in _right {
                                                        match right {
                                                            ast::CallExprArm::MemExprIden(x) => {
                                                                left = Box::new(ast::MemExpr {
                                                                    _span: (
                                                                        left.span().0,
                                                                        x.span().1,
                                                                    ),
                                                                    left,
                                                                    right: x,
                                                                    is_expr: false,
                                                                    is_namespace: false,
                                                                })
                                                            }
                                                            ast::CallExprArm::MemExprNs(x) => {
                                                                left = Box::new(ast::MemExpr {
                                                                    _span: (
                                                                        left.span().0,
                                                                        x.span().1,
                                                                    ),
                                                                    left,
                                                                    right: x,
                                                                    is_expr: false,
                                                                    is_namespace: true,
                                                                })
                                                            }
                                                            ast::CallExprArm::MemExpr(x) => {
                                                                left = Box::new(ast::MemExpr {
                                                                    _span: (
                                                                        left.span().0,
                                                                        x.span().1,
                                                                    ),
                                                                    left,
                                                                    right: x,
                                                                    is_expr: true,
                                                                    is_namespace: false,
                                                                })
                                                            }
                                                            ast::CallExprArm::CallExpr(x) => {
                                                                left = Box::new(ast::CallExpr {
                                                                    _span: (
                                                                        left.span().0,
                                                                        if let Some(last) =
                                                                            x.last()
                                                                        {
                                                                            last.span().1
                                                                        } else {
                                                                            left.span().0
                                                                        },
                                                                    ),
                                                                    callee: left,
                                                                    args: x,
                                                                })
                                                            }
                                                        };
                                                    }
                                                    left
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
        state.suppress_fail -= 1;
        res
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => memexpr(input, state, pos),
    }
}

fn callexpr_args(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<Vec<Box<dyn ast::Ast>>> {
    let choice_res = {
        match slice_eq(input, state, pos, "(") {
            Matched(pos, _) => {
                let seq_res = skip_white(input, state, pos);
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = expr(input, state, pos);
                        match seq_res {
                            Matched(pos, farg) => {
                                let seq_res = {
                                    let mut repeat_pos = pos;
                                    let mut repeat_value = vec![];
                                    loop {
                                        let pos = repeat_pos;
                                        let step_res = {
                                            let seq_res = skip_white(input, state, pos);
                                            match seq_res {
                                                Matched(pos, _) => {
                                                    let seq_res = slice_eq(input, state, pos, ",");
                                                    match seq_res {
                                                        Matched(pos, _) => {
                                                            let seq_res =
                                                                skip_white(input, state, pos);
                                                            match seq_res {
                                                                Matched(pos, _) => {
                                                                    let seq_res =
                                                                        expr(input, state, pos);
                                                                    match seq_res {
                                                                        Matched(pos, e) => {
                                                                            Matched(pos, e)
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
                                    Matched(pos, larg) => {
                                        let seq_res = skip_white(input, state, pos);
                                        match seq_res {
                                            Matched(pos, _) => {
                                                let seq_res = slice_eq(input, state, pos, ")");
                                                match seq_res {
                                                    Matched(pos, _) => Matched(pos, {
                                                        let mut args = vec![farg];
                                                        for arg in larg {
                                                            args.push(arg);
                                                        }
                                                        args
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
        Failed => match slice_eq(input, state, pos, "(") {
            Matched(pos, _) => {
                let seq_res = skip_white(input, state, pos);
                match seq_res {
                    Matched(pos, _) => {
                        let seq_res = slice_eq(input, state, pos, ")");
                        match seq_res {
                            Matched(pos, _) => Matched(pos, vec![]),
                            Failed => Failed,
                        }
                    }
                    Failed => Failed,
                }
            }
            Failed => Failed,
        },
    }
}

// Nota: Debo separar las llamadas normales de las de modulo (. / ::)
fn callexpr_arm(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<ast::CallExprArm> {
    let pos = if let Matched(pos, _) = white(input, state, pos) {
        pos
    } else {
        unreachable!("Rust never go here, Jajaja!");
    };

    let choice_res = {
        // Foo.method
        /*
        Esto puede ser optimizado ya que las primeras parte se repiten
        */
        match slice_eq(input, state, pos, ".") {
            Matched(pos, _) => {
                let seq_res = white(input, state, pos);
                match seq_res {
                    Matched(ps, _) => match word(input, state, ps) {
                        Matched(pe, id) => Matched(pos, {
                            ast::CallExprArm::MemExprIden(
                                boxed!(Identifier, ps, pe, val: id) as Box<dyn ast::Ast>
                            )
                        }),
                        Failed => Failed,
                    },
                    Failed => Failed,
                }
            }
            Failed => Failed,
        }
    };
    // Foo::method
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                match slice_eq(input, state, pos, "::") {
                    Matched(pos, _) => match white(input, state, pos) {
                        Matched(ps, _) => match word(input, state, pos) {
                            Matched(pe, id) => Matched(pe, {
                                ast::CallExprArm::MemExprNs(
                                    boxed!(Identifier, ps, pe, val: id) as Box<dyn ast::Ast>
                                )
                            }),
                            Failed => Failed,
                        },
                        Failed => Failed,
                    },
                    Failed => Failed,
                }
            };
            // [1,2,3]
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    let choice_res = {
                        match slice_eq(input, state, pos, "[") {
                            Matched(pos, _) => match skip_white(input, state, pos) {
                                Matched(pos, _) => match expr(input, state, pos) {
                                    Matched(pos, e) => match skip_white(input, state, pos) {
                                        Matched(pos, _) => {
                                            match slice_eq(input, state, pos, "]") {
                                                Matched(pos, _) => Matched(pos, {
                                                    let is_expr = e
                                                        .as_any()
                                                        .downcast_ref::<ast::StrLiteral>()
                                                        .is_none();
                                                    if is_expr {
                                                        ast::CallExprArm::MemExpr(e)
                                                    } else {
                                                        ast::CallExprArm::MemExprIden(e)
                                                    }
                                                }),
                                                Failed => Failed,
                                            }
                                        }
                                        Failed => Failed,
                                    },
                                    Failed => Failed,
                                },
                                Failed => Failed,
                            },
                            Failed => Failed,
                        }
                    };
                    // callexpr_args
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => match callexpr_args(input, state, pos) {
                            Matched(pos, args) => Matched(pos, ast::CallExprArm::CallExpr(args)),

                            Failed => Failed,
                        },
                    }
                }
            }
        }
    }
}

fn memexpr(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Box<dyn ast::Ast>> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            match unary_expr(input, state, pos) {
                Matched(pos, _left) => {
                    let seq_res = {
                        let mut repeat_pos = pos;
                        let mut repeat_value = vec![];
                        loop {
                            let pos = repeat_pos;
                            let step_res = memexpr_arm(input, state, pos);
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
                        if !repeat_value.is_empty() {
                            Matched(repeat_pos, repeat_value)
                        } else {
                            Failed
                        }
                    };
                    match seq_res {
                        Matched(pos, _right) => {
                            let seq_res = Matched(pos, pos);
                            match seq_res {
                                Matched(pos, _) => Matched(pos, {
                                    let mut left = _left;
                                    for right in _right {
                                        left = Box::new(ast::MemExpr {
                                            _span: (left.span().0, right.0.span().1),
                                            left,
                                            right: right.0,
                                            is_expr: right.1,
                                            is_namespace: right.2,
                                        });
                                    }
                                    left
                                }),
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
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => unary_expr(input, state, pos),
    }
}

fn memexpr_arm(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<(
    Box<dyn ast::Ast>,
    bool, /* is_expr */
    bool, /* is_namespace */
)> {
    let choice_res = {
        match white(input, state, pos) {
            Matched(pos, _) => match slice_eq(input, state, pos, ".") {
                Matched(pos, _) => match white(input, state, pos) {
                    Matched(ps, _) => match word(input, state, ps) {
                        Matched(pe, id) => Matched(pe, {
                            (
                                boxed!(Identifier, ps, pe, val: id) as Box<dyn ast::Ast>,
                                false,
                                false,
                            )
                        }),
                        Failed => Failed,
                    },
                    Failed => Failed,
                },
                Failed => Failed,
            },
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                match white(input, state, pos) {
                    Matched(pos, _) => match slice_eq(input, state, pos, "::") {
                        Matched(pos, _) => match white(input, state, pos) {
                            Matched(ps, _) => match word(input, state, ps) {
                                Matched(pe, id) => Matched(pe, {
                                    (
                                        boxed!(Identifier, ps, pe, val: id) as Box<dyn ast::Ast>,
                                        false,
                                        true,
                                    )
                                }),
                                Failed => Failed,
                            },

                            Failed => Failed,
                        },
                        Failed => Failed,
                    },
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => match white(input, state, pos) {
                    Matched(pos, _) => match slice_eq(input, state, pos, "[") {
                        Matched(pos, _) => match skip_white(input, state, pos) {
                            Matched(pos, _) => match expr(input, state, pos) {
                                Matched(pos, e) => match skip_white(input, state, pos) {
                                    Matched(pos, _) => match slice_eq(input, state, pos, "]") {
                                        Matched(pos, _) => Matched(pos, {
                                            let is_expr = e
                                                .as_any()
                                                .downcast_ref::<ast::StrLiteral>()
                                                .is_none();
                                            (e, is_expr, false)
                                        }),
                                        Failed => Failed,
                                    },
                                    Failed => Failed,
                                },
                                Failed => Failed,
                            },
                            Failed => Failed,
                        },
                        Failed => Failed,
                    },
                    Failed => Failed,
                },
            }
        }
    }
}
