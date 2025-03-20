use std::borrow::Cow;

use super::{ParseQueryError, Result, Value};
pub fn parse_string<'b>(input: &'b str) -> Result<'b, Cow<'b, str>> {
    match input.chars().next() {
        None => return Err(ParseQueryError::MalformedString(input)),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(input)),
        Some(_) => (),
    }

    match input.chars().last() {
        None => unreachable!(),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(input)),
        Some(_) => (),
    }
    if input.chars().any(|c| c == '\\') {
        Ok(Cow::Owned(unescape_string(input)?))
    } else {
        Ok(Cow::Borrowed(&input[1..input.len() - 1]))
    }
}
#[derive(Clone, Copy)]
enum State {
    None,
    Escape,
    Ascii(u8, u32),
    EnterUnicode,
    Unicode(u32, u32),
}
struct UnescapeString {
    state: State,
    output: String,
}
impl UnescapeString {
    pub fn new(capacity: usize) -> Self {
        Self {
            state: State::None,
            output: String::with_capacity(capacity),
        }
    }
    pub fn push_char(&mut self, c: char) {
        self.output.push(c);
        self.change_state(State::None);
    }
    pub fn change_state(&mut self, state: State) {
        self.state = state;
    }
}
pub fn unescape_string<'a>(input: &'a str) -> Result<'a, String> {
    let mut state = UnescapeString::new(input.len());
    for c in input.chars() {
        match state.state {
            State::None => {
                if c == '\\' {
                    state.change_state(State::Escape);
                }
            }
            State::Escape => {
                match c {
                    'n' => state.push_char('\n'),
                    't' => state.push_char('\t'),
                    'r' => state.push_char('\r'),
                    // '0' => state.push_char('\0'),
                    '\\' => state.push_char('\\'),
                    'x' => state.change_state(State::Ascii(0, 0)),
                    'u' => state.change_state(State::EnterUnicode),
                    _ => todo!("error"),
                }
            }
            State::Ascii(mut codepoint, mut len) => {
                codepoint = (codepoint << 4)
                    + c.to_digit(16).ok_or_else(|| todo!("error not hex digit"))? as u8;
                len += 1;
                if len == 2 {
                    state.push_char(codepoint.into());
                } else {
                    state.change_state(State::Ascii(codepoint, len));
                }
            }
            State::Unicode(mut codepoint, mut len) => {
                if c == '}' {
                    state.push_char(
                        char::from_u32(codepoint)
                            .ok_or_else(|| todo!("error not a valid codepoint"))?,
                    );
                } else {
                    codepoint = (codepoint << 4)
                        + c.to_digit(16).ok_or_else(|| todo!("error not hex digit"))?;
                    len += 1;
                    state.change_state(State::Unicode(codepoint, len));
                    if len > 6 {
                        todo!("error codepoint out of bound")
                    }
                }
            }
            State::EnterUnicode => {
                if c != '{' {
                    todo!("error {{ expected");
                }
                state.change_state(State::Unicode(0, 0));
            }
        }
    }
    Ok(state.output)
}

pub fn parse_alphanumeric<'a>(input: &'a str) -> Result<'a, Value<'a>> {
    enum Kind {
        Int(u32),
        Float,
        Str,
    }
    use Kind::*;
    let (sign, input2) = match input.chars().next() {
        None => unreachable!(),
        Some('-') => (-1, &input[1..]),
        Some('+') => (1, &input[1..]),
        Some(_) => (1, input),
    };

    let kind = if input2.starts_with("0x") {
        Int(16)
    } else if input2.starts_with("0o") {
        Int(8)
    } else if input2.starts_with("0b") {
        Int(2)
    } else if input2.chars().next().map(char::is_numeric) == Some(true) {
        match input2.chars().filter(|c| *c == '.').count() {
            0 => Int(10),
            1 => Float,
            _ => return Err(ParseQueryError::MalformedNumber(input)),
        }
    } else {
        Str
    };
    let result = match kind {
        Int(radix) => i64::from_str_radix(&input2[(2 * ((radix != 10) as usize))..], radix)
            .map(|v| Value::Integer(v * sign))
            .map_err(|_| ParseQueryError::MalformedNumber(input))?,
        Float => input
            .parse()
            .map(Value::FloatingPoing)
            .map_err(|_| ParseQueryError::MalformedNumber(input))?,
        Str => Value::Str(input),
    };
    Ok(result)
}
