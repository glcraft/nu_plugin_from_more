use std::{collections::HashMap, fmt::Display};

use crate::lexer::{Lexer, TokenType};

#[derive(Clone, PartialEq, Debug)]
pub enum Selector<'a> {
    /// "<name>" Node with a name
    Named(&'a str),
    /// "*" Any nodes in the current scope
    Any,
    /// "/" Root node
    Root,
    /// "//" Nodes starting anywhere in the doc
    Anywhere,
    /// ".." Parent node
    Parent,
    /// entry selector
    Entries(Entries<'a>),
    /// ranged or indexed selection
    Ranged(i32, Option<i32>),
}

impl<'a> Display for Selector<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(s) => write!(f, "{}", s),
            Self::Any => write!(f, "*"),
            Self::Root => write!(f, "/"),
            Self::Anywhere => write!(f, "//"),
            Self::Parent => write!(f, ".."),
            Self::Entries(e) => write!(f, "{}", e),
            Self::Ranged(beg, None) => write!(f, "{{{0}}}", beg),
            Self::Ranged(beg, Some(end)) => write!(f, "{{{0}..{1}}}", beg, end),
            _ => todo!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value<'a> {
    /// String
    Str(&'a str),
    /// Integer
    Integer(i64),
    /// Floating point
    FloatingPoing(f64),
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", s),
            Self::Integer(i) => write!(f, "{}", i),
            Self::FloatingPoing(fp) => write!(f, "{}", fp),
        }
    }
}

type Selectors<'a> = Vec<Selector<'a>>;

#[derive(Clone, PartialEq, Debug)]
pub enum EntryKind<'a> {
    Argument { position: u64, value: Value<'a> },
    Property { name: &'a str, value: Value<'a> },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Entries<'a>(Vec<EntryKind<'a>>);

impl<'a> Display for Entries<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> Entries<'a> {
    pub fn new() -> Self {
        Default::default()
    }
    fn parse_lexer(lexer: &mut impl Iterator<Item = TokenType<'a>>) -> Result<'a, Self> {
        let mut entries = vec![];
        let mut arg_pos = 0;
        let mut prop_name = None;
        let mut is_unnamed_arg = false;
        while let Some(token) = lexer.next() {
            let value = match token {
                TokenType::LeaveSquareBracket => break,
                TokenType::Alphanumeric(s) => {
                    is_unnamed_arg = true;
                    Path::parse_alphanumeric(s)?
                }
                TokenType::String(s) => {
                    is_unnamed_arg = true;
                    Value::Str(Path::parse_string(s)?)
                }
                TokenType::Equal => {
                    if prop_name.is_some() {
                        return Err(ParseQueryError::DoubleEqual);
                    }
                    let Some(EntryKind::Argument { position: _, value }) = entries.pop() else {
                        return Err(ParseQueryError::MissingEntryIdentifier);
                    };
                    if !is_unnamed_arg {
                        return Err(ParseQueryError::MissingEntryIdentifier);
                    }
                    prop_name = Some(value);
                    arg_pos -= 1;
                    is_unnamed_arg = false;
                    continue;
                }
                t => return Err(ParseQueryError::UnexpectedToken(t)),
            };
            match prop_name {
                Some(Value::Str(name)) => {
                    entries.push(EntryKind::Property { name, value });
                    prop_name = None;
                    is_unnamed_arg = false;
                }
                Some(Value::Integer(position)) => {
                    entries.push(EntryKind::Argument {
                        position: position as _,
                        value,
                    });
                    prop_name = None;
                    is_unnamed_arg = false;
                }
                Some(s @ Value::FloatingPoing(_)) => {
                    return Err(ParseQueryError::UnexpectedEntryIdentifier(s))
                }
                None => {
                    entries.push(EntryKind::Argument {
                        position: arg_pos,
                        value,
                    });
                    arg_pos += 1;
                }
            }
        }
        if prop_name.is_some() {
            return Err(ParseQueryError::MissingEntryValue);
        }
        Ok(Entries(entries))
    }
}
impl<'a> Default for Entries<'a> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Path<'a> {
    nodes: Selectors<'a>,
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum ParseQueryError<'a> {
    #[error("unexpected token: {0}")]
    UnexpectedToken(TokenType<'a>),
    #[error("malformed string (missing \"): {0}")]
    MalformedString(&'a str),
    #[error("malformed number: {0}")]
    MalformedNumber(&'a str),
    #[error("double equal in entries")]
    DoubleEqual,
    #[error("missing entry value after an equal")]
    MissingEntryValue,
    #[error("entry must be identified by string (property name) or integer number (argument index), got {0}")]
    UnexpectedEntryIdentifier(Value<'a>),
    #[error("missing entry identifier")]
    MissingEntryIdentifier,
    #[error("markers like root or anywhere can't have entries")]
    EntriesOnMarker,
    #[error("expected a node, but got something else")]
    NotANode,
}

type Result<'a, T> = std::result::Result<T, ParseQueryError<'a>>;

impl<'a> Path<'a> {
    pub fn parse(input: &'a str) -> Result<'a, Self> {
        let mut lexer = Lexer::from(input).peekable();
        let mut nodes = Vec::new();

        loop {
            let Some(token) = lexer.next() else {
                break;
            };
            let selector = match token {
                TokenType::Slash => {
                    if nodes.is_empty() {
                        Selector::Root
                    } else {
                        continue;
                    }
                }
                TokenType::DoubleSlash => {
                    if nodes.is_empty() {
                        Selector::Anywhere
                    } else {
                        return Err(ParseQueryError::UnexpectedToken(token));
                    }
                }
                TokenType::Star => Selector::Any,
                TokenType::DoublePoint => Selector::Parent,
                TokenType::String(s) => Selector::Named(Self::parse_string(s)?),
                TokenType::Alphanumeric(s) => {
                    let value = Self::parse_alphanumeric(s)?;
                    let Value::Str(name) = value else {
                        return Err(ParseQueryError::NotANode);
                    };
                    Selector::Named(name)
                }
                TokenType::EnterSquareBracket => {
                    Selector::Entries(Entries::parse_lexer(&mut lexer)?)
                }
                TokenType::EnterCurlyBracket => todo!(),
                _ => return Err(ParseQueryError::UnexpectedToken(token)),
            };
            nodes.push(selector);
        }

        return Ok(Self { nodes });
    }
    fn parse_string<'b>(s: &'b str) -> Result<'b, &'b str> {
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
    fn parse_alphanumeric(input: &'a str) -> Result<'a, Value<'a>> {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, TokenType},
        parser::{Entries, EntryKind, ParseQueryError, Path, Selector, Value},
        util::hashmap,
    };

    #[test]
    fn parser_strings() {
        fn test_string(input: &str, output: Result<&str, ParseQueryError>) {
            let mut lexer = Lexer::from(input);
            let token = lexer.next();
            assert_eq!(token, Some(TokenType::String(input)));
            assert_eq!(Path::parse_string(input), output);
        }
        test_string("\"hello\"", Ok("hello"));
        test_string("\"hello world\"", Ok("hello world"));
        test_string(
            "\"hello world",
            Err(ParseQueryError::MalformedString("\"hello world")),
        );
    }
    #[test]
    fn entries() {
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"1 2 3"#)),
            Ok(Entries(vec![
                EntryKind::Argument {
                    position: 0,
                    value: Value::Integer(1)
                },
                EntryKind::Argument {
                    position: 1,
                    value: Value::Integer(2)
                },
                EntryKind::Argument {
                    position: 2,
                    value: Value::Integer(3)
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"1 abc 3.14"#)),
            Ok(Entries(vec![
                EntryKind::Argument {
                    position: 0,
                    value: Value::Integer(1)
                },
                EntryKind::Argument {
                    position: 1,
                    value: Value::Str("abc")
                },
                EntryKind::Argument {
                    position: 2,
                    value: Value::FloatingPoing(3.14)
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"a=b c=d"#)),
            Ok(Entries(vec![
                EntryKind::Property {
                    name: "a",
                    value: Value::Str("b")
                },
                EntryKind::Property {
                    name: "c",
                    value: Value::Str("d")
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"name1=123 name2=abc name3=3.14"#)),
            Ok(Entries(vec![
                EntryKind::Property {
                    name: "name1",
                    value: Value::Integer(123)
                },
                EntryKind::Property {
                    name: "name2",
                    value: Value::Str("abc")
                },
                EntryKind::Property {
                    name: "name3",
                    value: Value::FloatingPoing(3.14)
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"1=123 2=abc 3=3.14"#)),
            Ok(Entries(vec![
                EntryKind::Argument {
                    position: 1,
                    value: Value::Integer(123)
                },
                EntryKind::Argument {
                    position: 2,
                    value: Value::Str("abc")
                },
                EntryKind::Argument {
                    position: 3,
                    value: Value::FloatingPoing(3.14)
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(
                r#"1 2 3 a=4 b = 5 "p r o p"="v a l u e" 6 10=7"#
            )),
            Ok(Entries(vec![
                EntryKind::Argument {
                    position: 0,
                    value: Value::Integer(1)
                },
                EntryKind::Argument {
                    position: 1,
                    value: Value::Integer(2)
                },
                EntryKind::Argument {
                    position: 2,
                    value: Value::Integer(3)
                },
                EntryKind::Property {
                    name: "a",
                    value: Value::Integer(4)
                },
                EntryKind::Property {
                    name: "b",
                    value: Value::Integer(5)
                },
                EntryKind::Property {
                    name: "p r o p",
                    value: Value::Str("v a l u e")
                },
                EntryKind::Argument {
                    position: 3,
                    value: Value::Integer(6)
                },
                EntryKind::Argument {
                    position: 10,
                    value: Value::Integer(7)
                },
            ]))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"1="#)),
            Err(ParseQueryError::MissingEntryValue)
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"name="#)),
            Err(ParseQueryError::MissingEntryValue)
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"3.1=abc"#)),
            Err(ParseQueryError::UnexpectedEntryIdentifier(
                Value::FloatingPoing(3.1)
            ))
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"=abc"#)),
            Err(ParseQueryError::MissingEntryIdentifier)
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"1=abc =cba"#)),
            Err(ParseQueryError::MissingEntryIdentifier)
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"name=abc =cba"#)),
            Err(ParseQueryError::MissingEntryIdentifier)
        );
        assert_eq!(
            Entries::parse_lexer(&mut Lexer::from(r#"name=abc [ ]"#)),
            Err(ParseQueryError::UnexpectedToken(
                TokenType::EnterSquareBracket
            ))
        );
    }
    #[test]
    fn path_named_one() {
        assert_eq!(
            Path::parse("node_name"),
            Ok(Path {
                nodes: vec![Selector::Named("node_name")]
            })
        );
    }

    #[test]
    fn path_named_multi() {
        assert_eq!(
            Path::parse("node1/node2"),
            Ok(Path {
                nodes: vec![Selector::Named("node1"), Selector::Named("node2")]
            })
        );
    }

    #[test]
    fn path_named_strings() {
        assert_eq!(
            Path::parse(r#""node 1"/"node 2""#),
            Ok(Path {
                nodes: vec![Selector::Named("node 1"), Selector::Named("node 2")]
            })
        );
    }
    #[test]
    fn path_ident_root() {
        assert_eq!(
            Path::parse(r#"/node1"#),
            Ok(Path {
                nodes: vec![Selector::Root, Selector::Named("node1")]
            })
        );
        assert_eq!(
            Path::parse(r#"/node1/node2"#),
            Ok(Path {
                nodes: vec![
                    Selector::Root,
                    Selector::Named("node1"),
                    Selector::Named("node2")
                ]
            })
        );
    }

    #[test]
    fn path_ident_anywhere() {
        assert_eq!(
            Path::parse(r#"//node1"#),
            Ok(Path {
                nodes: vec![Selector::Anywhere, Selector::Named("node1")]
            })
        );
    }

    #[test]
    fn path_ident_any() {
        assert_eq!(
            Path::parse(r#"/*/node1"#),
            Ok(Path {
                nodes: vec![Selector::Root, Selector::Any, Selector::Named("node1")]
            })
        );
    }
    #[test]
    fn path_ident_parents() {
        assert_eq!(
            Path::parse(r#"/../node1"#),
            Ok(Path {
                nodes: vec![Selector::Root, Selector::Parent, Selector::Named("node1")]
            })
        );
    }
    #[test]
    fn alphanum() {
        use {ParseQueryError::*, Value::*};
        let parse = Path::parse_alphanumeric;
        assert_eq!(parse("123"), Ok(Integer(123)));
        assert_eq!(parse("-123"), Ok(Integer(-123)));
        assert_eq!(parse("0x123"), Ok(Integer(0x123)));
        assert_eq!(parse("0b101"), Ok(Integer(0b101)));
        assert_eq!(parse("0o123"), Ok(Integer(0o123)));
        assert_eq!(parse("-0x123"), Ok(Integer(-0x123)));
        assert_eq!(parse("-0b101"), Ok(Integer(-0b101)));
        assert_eq!(parse("-0o123"), Ok(Integer(-0o123)));
        assert_eq!(parse("1.23"), Ok(FloatingPoing(1.23)));
        assert_eq!(parse("-1.23"), Ok(FloatingPoing(-1.23)));
        assert_eq!(parse("1.2.3"), Err(MalformedNumber("1.2.3")));
        assert_eq!(parse("-1.2.3"), Err(MalformedNumber("-1.2.3")));
        assert_eq!(parse("1c0"), Err(MalformedNumber("1c0")));
        assert_eq!(parse("-1c0"), Err(MalformedNumber("-1c0")));
        assert_eq!(parse("abc"), Ok(Str("abc")));
        assert_eq!(parse("-abc"), Ok(Str("-abc")));
        assert_eq!(parse("a1c"), Ok(Str("a1c")));
        assert_eq!(parse("-a1c"), Ok(Str("-a1c")));
    }
    // #[test]
}
