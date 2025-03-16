use std::collections::HashMap;

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
    /// Integer
    Integer(i64),
    /// Floating point
    FloatingPoing(f64),
}

type Selectors<'a> = Vec<Selector<'a>>;

#[derive(Clone, PartialEq, Eq, Debug)]
struct Entries<'a> {
    arguments: Vec<&'a str>,
    properties: HashMap<&'a str, &'a str>,
}

impl<'a> Entries<'a> {
    pub fn new() -> Self {
        Default::default()
    }
    fn parse_lexer(lexer: &mut impl Iterator<Item = TokenType<'a>>) -> Result<'a, Self> {
        let mut args = vec![];
        let mut props = HashMap::new();
        let mut prop_name = None;
        while let Some(token) = lexer.next() {
            let val = match token {
                TokenType::LeaveSquareBracket => break,
                TokenType::Alphanumeric(s) => s,
                TokenType::String(s) => Path::parse_string(s)?,
                TokenType::Equal => {
                    if prop_name.is_some() {
                        return Err(ParseQueryError::DoubleEqual);
                    }
                    let Some(s) = args.pop() else {
                        return Err(ParseQueryError::MissingPropertyName);
                    };
                    prop_name = Some(s);
                    continue;
                }
                t => return Err(ParseQueryError::UnexpectedToken(t)),
            };
            match prop_name {
                Some(name) => {
                    props.insert(name, val);
                    prop_name = None;
                }
                None => {
                    args.push(val);
                }
            }
        }
        Ok(Entries {
            arguments: args,
            properties: props,
        })
    }
}
impl<'a> Default for Entries<'a> {
    fn default() -> Self {
        Self {
            arguments: Vec::new(),
            properties: HashMap::new(),
        }
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
    #[error("missing property name")]
    MissingPropertyName,
    #[error("markers like root or anywhere can't have entries")]
    EntriesOnMarker,
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
                TokenType::Alphanumeric(s) => todo!(),
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
    fn parse_alphanumeric(input: &'a str) -> Result<'a, Selector<'a>> {
        let (n_points, hexa, other) = input.chars().fold(
            (0u32, false, false),
            |acc @ (n_points, hexa, other), c| match c {
                '-' | '0'..'9' => acc,
                '.' => (n_points + 1, hexa, other),
                'a'..'f' | 'A'..'F' => (n_points, true, other),
                _ => (n_points, hexa, true),
            },
        );
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, TokenType},
        parser::{Entries, ParseQueryError, Path, Selector},
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
    fn parser_entries() {
        let mut lexer = Lexer::from(r#"[1 2 3 a=4 b = 5 "p r o p"="v a l u e" 6]"#);
        assert_eq!(lexer.next(), Some(TokenType::EnterSquareBracket));
        assert_eq!(
            Entries::parse_lexer(&mut lexer),
            Ok(Entries {
                arguments: vec!["1", "2", "3", "6"],
                properties: hashmap! {
                    "a" => "4",
                    "b" => "5",
                    "p r o p" => "v a l u e"
                }
            })
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
        use {ParseQueryError::*, Selector::*};
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
        assert_eq!(parse("abc"), Ok(Named("abc")));
        assert_eq!(parse("-abc"), Ok(Named("-abc")));
    }
    // #[test]
}
