//! Parser implementation
#![allow(clippy::redundant_field_names)]

mod expressions;
mod statements;
mod token;
mod values;

use self::RuleResult::{Failed, Matched};
use crate::ast;
use statements::statement_program;
use token::skip_white;

#[macro_export]
macro_rules! boxed {
    ($x:ident, $ps:expr, $pe:expr, $($key:ident: $value:expr),*) => (
        Box::new(ast::$x {
            _span: ($ps, $pe),
            $($key: $value),*
        })
    )
}

fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

pub(super) fn char_range_at(s: &str, pos: usize) -> (char, usize) {
    let c = &s[pos..].chars().next().unwrap();
    let next_pos = pos + c.len_utf8();
    (*c, next_pos)
}

#[derive(Clone)]
pub(super) enum RuleResult<T> {
    Matched(usize, T),
    Failed,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub expected: std::collections::HashSet<&'static str>,
}

pub type ParseResult<T> = Result<T, ParseError>;

impl std::fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "error at {}:{}: expected ", self.line, self.column)?;
        if self.expected.is_empty() {
            write!(fmt, "EOF")?;
        } else if self.expected.len() == 1 {
            write!(
                fmt,
                "`{}`",
                escape_default(self.expected.iter().next().unwrap())
            )?;
        } else {
            let mut iter = self.expected.iter();
            write!(fmt, "one of `{}`", escape_default(iter.next().unwrap()))?;
            for elem in iter {
                write!(fmt, ", `{}`", escape_default(elem))?;
            }
        }
        Ok(())
    }
}
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        "parse error"
    }
}

#[inline(always)]
pub(super) fn slice_eq(
    input: &str,
    state: &mut ParseState,
    pos: usize,
    m: &'static str,
) -> RuleResult<()> {
    let l = m.len();
    if input.len() >= pos + l && &input.as_bytes()[pos..pos + l] == m.as_bytes() {
        Matched(pos + l, ())
    } else {
        state.mark_failure(pos, m)
    }
}

// I will use it for x | X
#[allow(dead_code)]
fn slice_eq_case_insensitive(
    input: &str,
    state: &mut ParseState,
    pos: usize,
    m: &'static str,
) -> RuleResult<()> {
    let mut used = 0usize;
    let mut input_iter = input[pos..].chars().flat_map(|x| x.to_uppercase());
    for m_char_upper in m.chars().flat_map(|x| x.to_uppercase()) {
        used += m_char_upper.len_utf8();
        let input_char_result = input_iter.next();
        if input_char_result.is_none() || input_char_result.unwrap() != m_char_upper {
            return state.mark_failure(pos, m);
        }
    }
    Matched(pos + used, ())
}

#[inline(always)]
pub(super) fn any_char(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    if input.len() > pos {
        let (_, next) = char_range_at(input, pos);
        Matched(next, ())
    } else {
        state.mark_failure(pos, "<character>")
    }
}

fn pos_to_line(input: &str, pos: usize) -> (usize, usize) {
    let before = &input[..pos];
    let line = before.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1;
    let col = before.chars().rev().take_while(|&c| c != '\n').count() + 1;
    (line, col)
}

pub(super) struct ParseState {
    max_err_pos: usize,
    suppress_fail: usize,
    expected: std::collections::HashSet<&'static str>,
}

impl ParseState {
    fn new() -> ParseState {
        ParseState {
            max_err_pos: 0,
            suppress_fail: 0,
            expected: std::collections::HashSet::new(),
        }
    }

    pub(super) fn mark_failure(&mut self, pos: usize, expected: &'static str) -> RuleResult<()> {
        if self.suppress_fail == 0 {
            if pos > self.max_err_pos {
                self.max_err_pos = pos;
                self.expected.clear();
            }
            if pos == self.max_err_pos {
                self.expected.insert(expected);
            }
        }
        Failed
    }
}

fn program_prologue(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<()> {
    let seq_res = slice_eq(input, state, pos, "#!");
    match seq_res {
        Matched(pos, _) => {
            let mut repeat = pos;
            loop {
                let pos = repeat;
                let step_res = if input.len() > pos {
                    match char_range_at(input, pos) {
                        ('\n', _) => state.mark_failure(pos, "[^\n]"),
                        (_, next) => Matched(next, ()),
                    }
                } else {
                    state.mark_failure(pos, "[^\n]")
                };
                match step_res {
                    Matched(newpos, _) => {
                        repeat = newpos;
                    }
                    Failed => {
                        break;
                    }
                }
            }
            Matched(repeat, ())
        }
        Failed => Matched(pos, ()), // Continue from Start
    }
}

fn start(input: &str, state: &mut ParseState, pos: usize) -> RuleResult<Vec<Box<dyn ast::Ast>>> {
    // TODO: Note that skip_white is not in the loop, I think it should be!
    // and save code...
    match program_prologue(input, state, pos) {
        Matched(pos, _) => match skip_white(input, state, pos) {
            Matched(pos, _) => {
                let mut repeat = pos;
                let mut repeat_value = vec![];
                loop {
                    let pos = repeat;
                    match statement_program(input, state, pos) {
                        Matched(newpos, value) => {
                            repeat = newpos;
                            repeat_value.push(value);
                        }
                        Failed => break,
                    }
                }
                Matched(repeat, repeat_value)
            }
            Failed => unreachable!(),
        },
        Failed => unreachable!(),
    }
}

pub fn parser_start(input: &str) -> ParseResult<Vec<Box<dyn ast::Ast>>> {
    let mut state = ParseState::new();
    match start(input, &mut state, 0) {
        Matched(pos, value) => {
            if pos == input.len() {
                return Ok(value);
            }
        }
        _ => {}
    }
    let (line, col) = pos_to_line(input, state.max_err_pos);
    Err(ParseError {
        line,
        column: col,
        offset: state.max_err_pos,
        expected: state.expected,
    })
}
