use super::{ParseQueryError, Result, Value};
pub fn parse_string<'b>(s: &'b str) -> Result<'b, &'b str> {
    match s.chars().next() {
        None => return Err(ParseQueryError::MalformedString(s)),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(s)),
        Some(_) => (),
    }

    match s.chars().last() {
        None => unreachable!(),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(s)),
        Some(_) => (),
    }
    // TODO: Convert escaped characters, output a Cow string
    return Ok(&s[1..s.len() - 1]);
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
