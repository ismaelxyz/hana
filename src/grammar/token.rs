use super::{
    any_char, char_range_at, slice_eq, slice_eq_case_insensitive, ParseState, RuleResult,
    RuleResult::*,
};

pub(super) fn int_literal(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<i64> {
    let choice_res = {
        let seq_res = slice_eq(input, state, pos, "0");
        match seq_res {
            Matched(pos, _) => match slice_eq_case_insensitive(input, state, pos, "x") {
                Matched(pos, _) => {
                    let seq_res = {
                        let str_start = pos;
                        match {
                            state.suppress_fail += 1;
                            let res = {
                                let mut repeat_pos = pos;
                                let mut repeat_value = vec![];
                                loop {
                                    let pos = repeat_pos;
                                    let step_res = if input.len() > pos {
                                        let (ch, next) = char_range_at(input, pos);
                                        match ch {
                                            '0'..='9' | 'a'..='f' | 'A'..='F' => Matched(next, ()),
                                            _ => state.mark_failure(pos, "[0-9a-fA-F]"),
                                        }
                                    } else {
                                        state.mark_failure(pos, "[0-9a-fA-F]")
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
                                if !repeat_value.is_empty() {
                                    Matched(repeat_pos, ())
                                } else {
                                    Failed
                                }
                            };
                            state.suppress_fail -= 1;
                            res
                        } {
                            Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                            Failed => Failed,
                        }
                    };
                    match seq_res {
                        Matched(pos, n) => Matched(pos, i64::from_str_radix(n, 16).unwrap()),
                        Failed => Failed,
                    }
                }
                Failed => Failed,
            },
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                let seq_res = slice_eq(input, state, pos, "0");
                match seq_res {
                    Matched(pos, _) => match slice_eq_case_insensitive(input, state, pos, "o") {
                        Matched(pos, _) => {
                            let seq_res = {
                                let str_start = pos;
                                match {
                                    state.suppress_fail += 1;
                                    let res = {
                                        let mut repeat_pos = pos;
                                        let mut repeat_value = vec![];
                                        loop {
                                            let pos = repeat_pos;
                                            let step_res = if input.len() > pos {
                                                let (ch, next) = char_range_at(input, pos);
                                                match ch {
                                                    '0'..='7' => Matched(next, ()),
                                                    _ => state.mark_failure(pos, "[0-7]"),
                                                }
                                            } else {
                                                state.mark_failure(pos, "[0-7]")
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
                                        if repeat_value.len() >= 1 {
                                            Matched(repeat_pos, ())
                                        } else {
                                            Failed
                                        }
                                    };
                                    state.suppress_fail -= 1;
                                    res
                                } {
                                    Matched(newpos, _) => {
                                        Matched(newpos, &input[str_start..newpos])
                                    }
                                    Failed => Failed,
                                }
                            };
                            match seq_res {
                                Matched(pos, n) => {
                                    Matched(pos, i64::from_str_radix(n, 8).unwrap())
                                }
                                Failed => Failed,
                            }
                        }
                        Failed => Failed,
                    },
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    let choice_res = {
                        let seq_res = slice_eq(input, state, pos, "0");
                        match seq_res {
                            Matched(pos, _) => {
                                match slice_eq_case_insensitive(input, state, pos, "b") {
                                    Matched(pos, _) => {
                                        let seq_res = {
                                            let str_start = pos;
                                            match {
                                                state.suppress_fail += 1;
                                                let res = {
                                                    let mut repeat_pos = pos;
                                                    let mut repeat_value = vec![];
                                                    loop {
                                                        let pos = repeat_pos;
                                                        let step_res = if input.len() > pos {
                                                            let (ch, next) =
                                                                char_range_at(input, pos);
                                                            match ch {
                                                                '0'..='1' => Matched(next, ()),
                                                                _ => state
                                                                    .mark_failure(pos, "[0-1]"),
                                                            }
                                                        } else {
                                                            state.mark_failure(pos, "[0-1]")
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
                                                    if repeat_value.len() >= 1 {
                                                        Matched(repeat_pos, ())
                                                    } else {
                                                        Failed
                                                    }
                                                };
                                                state.suppress_fail -= 1;
                                                res
                                            } {
                                                Matched(newpos, _) => {
                                                    Matched(newpos, &input[str_start..newpos])
                                                }
                                                Failed => Failed,
                                            }
                                        };
                                        match seq_res {
                                            Matched(pos, n) => Matched(pos, {
                                                i64::from_str_radix(n, 2).unwrap()
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
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => {
                            let choice_res = {
                                let seq_res = {
                                    let str_start = pos;
                                    match {
                                        state.suppress_fail += 1;
                                        let res = {
                                            let mut repeat_pos = pos;
                                            let mut repeat_value = vec![];
                                            loop {
                                                let pos = repeat_pos;
                                                let step_res = if input.len() > pos {
                                                    let (ch, next) = char_range_at(input, pos);
                                                    match ch {
                                                        '0'..='9' => Matched(next, ()),
                                                        _ => state.mark_failure(pos, "[0-9]"),
                                                    }
                                                } else {
                                                    state.mark_failure(pos, "[0-9]")
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
                                            if repeat_value.len() >= 1 {
                                                Matched(repeat_pos, ())
                                            } else {
                                                Failed
                                            }
                                        };
                                        state.suppress_fail -= 1;
                                        res
                                    } {
                                        Matched(newpos, _) => {
                                            Matched(newpos, &input[str_start..newpos])
                                        }
                                        Failed => Failed,
                                    }
                                };
                                match seq_res {
                                    Matched(pos, n) => Matched(pos, n.parse::<i64>().unwrap()),
                                    Failed => Failed,
                                }
                            };
                            match choice_res {
                                Matched(pos, value) => Matched(pos, value),
                                Failed => {
                                    state.mark_failure(pos, "integer literal");
                                    Failed
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn float_literal(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<f64> {
    let choice_res = {
        let seq_res = {
            let str_start = pos;
            match {
                state.suppress_fail += 1;
                let res = {
                    let choice_res = {
                        let seq_res = {
                            let mut repeat_pos = pos;
                            let mut repeat_value = vec![];
                            loop {
                                let pos = repeat_pos;
                                let step_res = if input.len() > pos {
                                    let (ch, next) = char_range_at(input, pos);
                                    match ch {
                                        '0'..='9' => Matched(next, ()),
                                        _ => state.mark_failure(pos, "[0-9]"),
                                    }
                                } else {
                                    state.mark_failure(pos, "[0-9]")
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
                            if repeat_value.len() >= 1 {
                                Matched(repeat_pos, ())
                            } else {
                                Failed
                            }
                        };
                        match seq_res {
                            Matched(pos, _) => {
                                let seq_res = slice_eq(input, state, pos, ".");
                                match seq_res {
                                    Matched(pos, _) => {
                                        let mut repeat_pos = pos;
                                        let mut repeat_value = vec![];
                                        loop {
                                            let pos = repeat_pos;
                                            let step_res = if input.len() > pos {
                                                let (ch, next) = char_range_at(input, pos);
                                                match ch {
                                                    '0'..='9' => Matched(next, ()),
                                                    _ => state.mark_failure(pos, "[0-9]"),
                                                }
                                            } else {
                                                state.mark_failure(pos, "[0-9]")
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
                                        if repeat_value.len() >= 1 {
                                            Matched(repeat_pos, ())
                                        } else {
                                            Failed
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
                                let seq_res = {
                                    let mut repeat_pos = pos;
                                    let mut repeat_value = vec![];
                                    loop {
                                        let pos = repeat_pos;
                                        let step_res = if input.len() > pos {
                                            let (ch, next) = char_range_at(input, pos);
                                            match ch {
                                                '0'..='9' => Matched(next, ()),
                                                _ => state.mark_failure(pos, "[0-9]"),
                                            }
                                        } else {
                                            state.mark_failure(pos, "[0-9]")
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
                                    if repeat_value.len() >= 1 {
                                        Matched(repeat_pos, ())
                                    } else {
                                        Failed
                                    }
                                };
                                match seq_res {
                                    Matched(pos, _) => slice_eq(input, state, pos, "."),
                                    Failed => Failed,
                                }
                            };
                            match choice_res {
                                Matched(pos, value) => Matched(pos, value),
                                Failed => {
                                    let seq_res = slice_eq(input, state, pos, ".");
                                    match seq_res {
                                        Matched(pos, _) => {
                                            let mut repeat_pos = pos;
                                            let mut repeat_value = vec![];
                                            loop {
                                                let pos = repeat_pos;
                                                let step_res = if input.len() > pos {
                                                    let (ch, next) = char_range_at(input, pos);
                                                    match ch {
                                                        '0'..='9' => Matched(next, ()),
                                                        _ => state.mark_failure(pos, "[0-9]"),
                                                    }
                                                } else {
                                                    state.mark_failure(pos, "[0-9]")
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
                                            if repeat_value.len() >= 1 {
                                                Matched(repeat_pos, ())
                                            } else {
                                                Failed
                                            }
                                        }
                                        Failed => Failed,
                                    }
                                }
                            }
                        }
                    }
                };
                state.suppress_fail -= 1;
                res
            } {
                Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                Failed => Failed,
            }
        };
        match seq_res {
            Matched(pos, n) => Matched(pos, n.parse::<f64>().unwrap()),
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "float literal");
            Failed
        }
    }
}

fn string_literal_escape(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let choice_res = {
        let seq_res = {
            let str_start = pos;
            match {
                state.suppress_fail += 1;
                let res = slice_eq(input, state, pos, "\\n");
                state.suppress_fail -= 1;
                res
            } {
                Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                Failed => Failed,
            }
        };
        match seq_res {
            Matched(pos, _) => Matched(pos, "\n".to_string()),
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let choice_res = {
                let seq_res = {
                    let str_start = pos;
                    match {
                        state.suppress_fail += 1;
                        let res = slice_eq(input, state, pos, "\\r");
                        state.suppress_fail -= 1;
                        res
                    } {
                        Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                        Failed => Failed,
                    }
                };
                match seq_res {
                    Matched(pos, _) => Matched(pos, "\r".to_string()),
                    Failed => Failed,
                }
            };
            match choice_res {
                Matched(pos, value) => Matched(pos, value),
                Failed => {
                    let choice_res = {
                        let seq_res = {
                            let str_start = pos;
                            match {
                                state.suppress_fail += 1;
                                let res = slice_eq(input, state, pos, "\\t");
                                state.suppress_fail -= 1;
                                res
                            } {
                                Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                                Failed => Failed,
                            }
                        };
                        match seq_res {
                            Matched(pos, _) => Matched(pos, "\t".to_string()),
                            Failed => Failed,
                        }
                    };
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => {
                            let seq_res = {
                                state.suppress_fail += 1;
                                let res = slice_eq(input, state, pos, "\\");
                                state.suppress_fail -= 1;
                                res
                            };
                            match seq_res {
                                Matched(pos, _) => {
                                    let seq_res = {
                                        let str_start = pos;
                                        match any_char(input, state, pos) {
                                            Matched(newpos, _) => {
                                                Matched(newpos, &input[str_start..newpos])
                                            }
                                            Failed => Failed,
                                        }
                                    };
                                    match seq_res {
                                        Matched(pos, c) => Matched(pos, c.to_string()),
                                        Failed => Failed,
                                    }
                                }
                                Failed => Failed,
                            }
                        }
                    }
                }
            }
        }
    }
}

fn string_literal_char(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let choice_res = string_literal_escape(input, state, pos);
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let seq_res = {
                let str_start = pos;
                match if input.len() > pos {
                    let (ch, next) = char_range_at(input, pos);
                    match ch {
                        '"' => state.mark_failure(pos, "[^\"]"),
                        _ => Matched(next, ()),
                    }
                } else {
                    state.mark_failure(pos, "[^\"]")
                } {
                    Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                    Failed => Failed,
                }
            };
            match seq_res {
                Matched(pos, s) => Matched(pos, s.to_owned()),
                Failed => Failed,
            }
        }
    }
}

fn string_literal_char_single(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<String> {
    let choice_res = string_literal_escape(input, state, pos);
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            let seq_res = {
                let str_start = pos;
                match if input.len() > pos {
                    let (ch, next) = char_range_at(input, pos);
                    match ch {
                        '\'' => state.mark_failure(pos, "[^']"),
                        _ => Matched(next, ()),
                    }
                } else {
                    state.mark_failure(pos, "[^']")
                } {
                    Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                    Failed => Failed,
                }
            };
            match seq_res {
                Matched(pos, s) => Matched(pos, s.to_owned()),
                Failed => Failed,
            }
        }
    }
}

pub(super) fn string_literal(
    input: &str,
    state: &mut ParseState,
    pos: usize,
) -> RuleResult<String> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            let seq_res = slice_eq(input, state, pos, "\"");
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = {
                        let mut repeat_pos = pos;
                        let mut repeat_value = vec![];
                        loop {
                            let pos = repeat_pos;
                            let step_res = string_literal_char(input, state, pos);
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
                            let seq_res = slice_eq(input, state, pos, "\"");
                            match seq_res {
                                Matched(pos, _) => Matched(pos, s.join("")),
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
        Failed => {
            let choice_res = {
                state.suppress_fail += 1;
                let res = {
                    let seq_res = slice_eq(input, state, pos, "'");
                    match seq_res {
                        Matched(pos, _) => {
                            let seq_res = {
                                let mut repeat_pos = pos;
                                let mut repeat_value = vec![];
                                loop {
                                    let pos = repeat_pos;
                                    let step_res = string_literal_char_single(input, state, pos);
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
                                    let seq_res = slice_eq(input, state, pos, "'");
                                    match seq_res {
                                        Matched(pos, _) => Matched(pos, s.join("")),
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
                Failed => {
                    state.mark_failure(pos, "string literal");
                    Failed
                }
            }
        }
    }
}

fn id_start(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let seq_res = {
        let str_start = pos;
        match if input.len() > pos {
            let (ch, next) = char_range_at(input, pos);
            match ch {
                'a'..='z' | 'A'..='Z' | '$' | '_' => Matched(next, ()),
                _ => state.mark_failure(pos, "[a-zA-Z$_]"),
            }
        } else {
            state.mark_failure(pos, "[a-zA-Z$_]")
        } {
            Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
            Failed => Failed,
        }
    };
    match seq_res {
        Matched(pos, c) => Matched(pos, c.to_string()),
        Failed => Failed,
    }
}

pub(super) fn id_chars(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let seq_res = {
        let str_start = pos;
        match if input.len() > pos {
            let (ch, next) = char_range_at(input, pos);
            match ch {
                'a'..='z' | 'A'..='Z' | '$' | '_' | '0'..='9' | '?' | '!' => Matched(next, ()),
                _ => state.mark_failure(pos, "[a-zA-Z$_0-9?!]"),
            }
        } else {
            state.mark_failure(pos, "[a-zA-Z$_0-9?!]")
        } {
            Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
            Failed => Failed,
        }
    };
    match seq_res {
        Matched(pos, c) => Matched(pos, c.to_string()),
        Failed => Failed,
    }
}

pub(super) fn word(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let choice_res = {
        let seq_res = {
            let str_start = pos;
            match {
                let seq_res = match id_start(input, state, pos) {
                    Matched(pos, _) => Matched(pos, ()),
                    Failed => Failed,
                };
                match seq_res {
                    Matched(pos, _) => {
                        let mut repeat_pos = pos;
                        loop {
                            let pos = repeat_pos;
                            let step_res = match id_chars(input, state, pos) {
                                Matched(pos, _) => Matched(pos, ()),
                                Failed => Failed,
                            };
                            match step_res {
                                Matched(newpos, _) => {
                                    repeat_pos = newpos;
                                }
                                Failed => {
                                    break;
                                }
                            }
                        }
                        Matched(repeat_pos, ())
                    }
                    Failed => Failed,
                }
            } {
                Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                Failed => Failed,
            }
        };
        match seq_res {
            Matched(pos, w) => Matched(pos, w.to_string()),
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "word");
            Failed
        }
    }
}

fn keyword(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let seq_res = {
        let choice_res = slice_eq(input, state, pos, "and");
        match choice_res {
            Matched(pos, value) => Matched(pos, value),
            Failed => {
                let choice_res = slice_eq(input, state, pos, "or");
                match choice_res {
                    Matched(pos, value) => Matched(pos, value),
                    Failed => {
                        let choice_res = slice_eq(input, state, pos, "not");
                        match choice_res {
                            Matched(pos, value) => Matched(pos, value),
                            Failed => {
                                let choice_res = slice_eq(input, state, pos, "begin");
                                match choice_res {
                                    Matched(pos, value) => Matched(pos, value),
                                    Failed => {
                                        let choice_res = slice_eq(input, state, pos, "end");
                                        match choice_res {
                                            Matched(pos, value) => Matched(pos, value),
                                            Failed => {
                                                let choice_res =
                                                    slice_eq(input, state, pos, "then");
                                                match choice_res {
                                                    Matched(pos, value) => Matched(pos, value),
                                                    Failed => {
                                                        let choice_res =
                                                            slice_eq(input, state, pos, "if");
                                                        match choice_res {
                                                            Matched(pos, value) => {
                                                                Matched(pos, value)
                                                            }
                                                            Failed => {
                                                                let choice_res = slice_eq(
                                                                    input, state, pos, "else",
                                                                );
                                                                match choice_res {
                                                                    Matched(pos, value) => {
                                                                        Matched(pos, value)
                                                                    }
                                                                    Failed => {
                                                                        let choice_res = slice_eq(
                                                                            input, state, pos,
                                                                            "while",
                                                                        );
                                                                        match choice_res {
                                                                            Matched(
                                                                                pos,
                                                                                value,
                                                                            ) => {
                                                                                Matched(pos, value)
                                                                            }
                                                                            Failed => {
                                                                                let choice_res =
                                                                                    slice_eq(
                                                                                        input,
                                                                                        state,
                                                                                        pos,
                                                                                        "for",
                                                                                    );
                                                                                match choice_res {
                                                                                    Matched(
                                                                                        pos,
                                                                                        value,
                                                                                    ) => Matched(
                                                                                        pos, value,
                                                                                    ),
                                                                                    Failed => {
                                                                                        let choice_res = slice_eq ( input , state , pos , "continue" ) ;
                                                                                        match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "break" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "fn" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "try" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "case" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "as" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "raise" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "in" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "of" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "match" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "func" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => { let choice_res = slice_eq ( input , state , pos , "return" ) ; match choice_res { Matched ( pos , value ) => Matched ( pos , value ) , Failed => slice_eq ( input , state , pos , "record" ) } } } } } } } } } } } } } } } } } } } } } } }
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
            }
        }
    };
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
}

pub(super) fn identifier(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<String> {
    let choice_res = {
        state.suppress_fail += 1;
        let res = {
            let seq_res = {
                state.suppress_fail += 1;
                let __assert_res = keyword(input, state, pos);
                state.suppress_fail -= 1;
                match __assert_res {
                    Failed => Matched(pos, ()),
                    Matched(..) => Failed,
                }
            };
            match seq_res {
                Matched(pos, _) => {
                    let seq_res = {
                        let str_start = pos;
                        match match word(input, state, pos) {
                            Matched(pos, _) => Matched(pos, ()),
                            Failed => Failed,
                        } {
                            Matched(newpos, _) => Matched(newpos, &input[str_start..newpos]),
                            Failed => Failed,
                        }
                    };
                    match seq_res {
                        Matched(pos, w) => Matched(pos, w.to_string()),
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
            state.mark_failure(pos, "identifier");
            Failed
        }
    }
}

fn single_line_comment(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let seq_res = slice_eq(input, state, pos, "//");
    match seq_res {
        Matched(pos, _) => {
            let mut repeat_pos = pos;
            loop {
                let pos = repeat_pos;
                let step_res = if input.len() > pos {
                    let (ch, next) = char_range_at(input, pos);
                    match ch {
                        '\n' => state.mark_failure(pos, "[^\n]"),
                        _ => Matched(next, ()),
                    }
                } else {
                    state.mark_failure(pos, "[^\n]")
                };
                match step_res {
                    Matched(newpos, _) => {
                        repeat_pos = newpos;
                    }
                    Failed => {
                        break;
                    }
                }
            }
            Matched(repeat_pos, ())
        }
        Failed => Failed,
    }
}

fn multiline_comment(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let seq_res = slice_eq(input, state, pos, "/*");
    match seq_res {
        Matched(pos, _) => {
            let seq_res = {
                let mut repeat_pos = pos;
                loop {
                    let pos = repeat_pos;
                    let step_res = {
                        let seq_res = {
                            state.suppress_fail += 1;
                            let __assert_res = slice_eq(input, state, pos, "*/");
                            state.suppress_fail -= 1;
                            match __assert_res {
                                Failed => Matched(pos, ()),
                                Matched(..) => Failed,
                            }
                        };
                        match seq_res {
                            Matched(pos, _) => any_char(input, state, pos),
                            Failed => Failed,
                        }
                    };
                    match step_res {
                        Matched(newpos, _) => {
                            repeat_pos = newpos;
                        }
                        Failed => {
                            break;
                        }
                    }
                }
                Matched(repeat_pos, ())
            };
            match seq_res {
                Matched(pos, _) => slice_eq(input, state, pos, "*/"),
                Failed => Failed,
            }
        }
        Failed => Failed,
    }
}

fn comment(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let choice_res = single_line_comment(input, state, pos);
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => multiline_comment(input, state, pos),
    }
}

pub(super) fn parse_s(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    state.suppress_fail += 1;
    let res = {
        let mut repeat_pos = pos;
        let mut repeat_value = vec![];
        loop {
            let pos = repeat_pos;
            let step_res = {
                let choice_res = comment(input, state, pos);
                match choice_res {
                    Matched(pos, value) => Matched(pos, value),
                    Failed => {
                        let choice_res = if input.len() > pos {
                            let (ch, next) = char_range_at(input, pos);
                            match ch {
                                ' ' | '\t' => Matched(next, ()),
                                _ => state.mark_failure(pos, "[ \t]"),
                            }
                        } else {
                            state.mark_failure(pos, "[ \t]")
                        };
                        match choice_res {
                            Matched(pos, value) => Matched(pos, value),
                            Failed => {
                                let seq_res = slice_eq(input, state, pos, "\\");
                                match seq_res {
                                    Matched(pos, _) => any_char(input, state, pos),
                                    Failed => Failed,
                                }
                            }
                        }
                    }
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
        if repeat_value.len() >= 1 {
            Matched(repeat_pos, ())
        } else {
            Failed
        }
    };
    state.suppress_fail -= 1;
    res
}

pub(super) fn white(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    state.suppress_fail += 1;
    let res = {
        let mut repeat_pos = pos;
        loop {
            let pos = repeat_pos;
            let step_res = {
                let choice_res = comment(input, state, pos);
                match choice_res {
                    Matched(pos, value) => Matched(pos, value),
                    Failed => {
                        let choice_res = if input.len() > pos {
                            let (ch, next) = char_range_at(input, pos);
                            match ch {
                                ' ' | '\t' => Matched(next, ()),
                                _ => state.mark_failure(pos, "[ \t]"),
                            }
                        } else {
                            state.mark_failure(pos, "[ \t]")
                        };
                        match choice_res {
                            Matched(pos, value) => Matched(pos, value),
                            Failed => {
                                let seq_res = slice_eq(input, state, pos, "\\");
                                match seq_res {
                                    Matched(pos, _) => any_char(input, state, pos),
                                    Failed => Failed,
                                }
                            }
                        }
                    }
                }
            };
            match step_res {
                Matched(newpos, _) => {
                    repeat_pos = newpos;
                }
                Failed => {
                    break;
                }
            }
        }
        Matched(repeat_pos, ())
    };
    state.suppress_fail -= 1;
    res
}

pub(super) fn skip_white(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    state.suppress_fail += 1;
    let res = {
        let mut repeat = pos;
        loop {
            let pos = repeat;
            let step_res = {
                match comment(input, state, pos) {
                    Matched(pos, value) => Matched(pos, value),
                    Failed => {
                        let choice_res = {
                            match slice_eq(input, state, pos, "\\") {
                                Matched(pos, _) => any_char(input, state, pos),
                                Failed => Failed,
                            }
                        };
                        match choice_res {
                            Matched(pos, value) => Matched(pos, value),
                            Failed => {
                                if input.len() > pos {
                                    let (ch, next) = char_range_at(input, pos);
                                    // White Space...
                                    match ch {
                                        '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}'
                                        | '\u{000D}' | '\u{0020}' | '\u{0085}' | '\u{200E}'
                                        | '\u{200F}' | '\u{2028}' | '\u{2029}' => {
                                            Matched(next, ())
                                        }
                                        _ => state.mark_failure(pos, "[ \t\r\n]"),
                                    }
                                } else {
                                    state.mark_failure(pos, "[ \t\r\n]")
                                }
                            }
                        }
                    }
                }
            };

            if let Matched(newpos, _) = step_res {
                repeat = newpos;
            } else {
                break;
            }
        }
        Matched(repeat, ())
    };
    state.suppress_fail -= 1;
    res
}

fn newline(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let choice_res = {
        let seq_res = white(input, state, pos);
        match seq_res {
            Matched(pos, _) => {
                state.suppress_fail += 1;
                let res = {
                    let choice_res = {
                        let mut repeat_pos = pos;
                        let mut repeat_value = vec![];
                        loop {
                            let pos = repeat_pos;
                            let step_res = slice_eq(input, state, pos, "\r\n");
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
                        if repeat_value.len() >= 1 {
                            Matched(repeat_pos, ())
                        } else {
                            Failed
                        }
                    };
                    match choice_res {
                        Matched(pos, value) => Matched(pos, value),
                        Failed => {
                            let choice_res = slice_eq(input, state, pos, ";");
                            match choice_res {
                                Matched(pos, value) => Matched(pos, value),
                                Failed => {
                                    let choice_res = {
                                        let mut repeat_pos = pos;
                                        let mut repeat_value = vec![];
                                        loop {
                                            let pos = repeat_pos;
                                            let step_res = slice_eq(input, state, pos, "\n");
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
                                        if repeat_value.len() >= 1 {
                                            Matched(repeat_pos, ())
                                        } else {
                                            Failed
                                        }
                                    };
                                    match choice_res {
                                        Matched(pos, value) => Matched(pos, value),
                                        Failed => {
                                            state.suppress_fail += 1;
                                            let __assert_res = any_char(input, state, pos);
                                            state.suppress_fail -= 1;
                                            match __assert_res {
                                                Failed => Matched(pos, ()),
                                                Matched(..) => Failed,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                };
                state.suppress_fail -= 1;
                res
            }
            Failed => Failed,
        }
    };
    match choice_res {
        Matched(pos, value) => Matched(pos, value),
        Failed => {
            state.mark_failure(pos, "newline");
            Failed
        }
    }
}

pub(super) fn eos(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    match white(input, state, pos) {
        Matched(pos, _) => newline(input, state, pos),
        Failed => Failed,
    }
}
