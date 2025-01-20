use std::collections::HashMap;

use crate::lexer::{Lexer, TokenType};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NodeIdentifier<'a> {
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node<'a> {
    ident: NodeIdentifier<'a>,
    entries: Entries<'a>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
struct Entries<'a> {
    arguments: Vec<&'a str>,
    properties: HashMap<&'a str, &'a str>,
}

impl<'a> Entries<'a> {
    pub fn new() -> Self {
        Default::default()
    }
    fn parse_lexer(lexer: &mut Lexer<'a>) -> Result<'a, Self> {
        let mut args = vec![];
        let mut props = HashMap::new();
        let mut prop_name = None;
        while let Some(token) = lexer.next() {
            let val = match token {
                TokenType::LeaveSquareBracket => break,
                TokenType::Alphanumeric(s) => s,
                t @ TokenType::String(_) => parse_string(t)?,
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Path<'a> {
    nodes: Vec<Node<'a>>,
}

#[derive(thiserror::Error, PartialEq, Eq, Debug)]
pub enum ParseQueryError<'a> {
    #[error("unexpected token: {0}")]
    UnexpectedToken(TokenType<'a>),
    #[error("malformed string (missing \"): {0}")]
    MalformedString(TokenType<'a>),
    #[error("double equal in entries")]
    DoubleEqual,
    #[error("missing property name")]
    MissingPropertyName,
}

type Result<'a, T> = std::result::Result<T, ParseQueryError<'a>>;

impl<'a> Path<'a> {
    pub fn parse(input: &'a str) -> Result<'a, Self> {
        let mut lexer = Lexer::from(input);
        let mut nodes = Vec::new();

        let first_node = match lexer.next() {
            None => return Ok(Path { nodes: vec![] }),
            Some(TokenType::Slash) => NodeIdentifier::Root,
            Some(TokenType::Star) => NodeIdentifier::Any,
            Some(TokenType::DoubleSlash) => NodeIdentifier::Anywhere,
            Some(TokenType::DoublePoint) => NodeIdentifier::Parent,
            Some(t @ TokenType::String(_)) => NodeIdentifier::Named(parse_string(t)?),
            Some(TokenType::Alphanumeric(s)) => NodeIdentifier::Named(s),
            Some(v) => return Err(ParseQueryError::UnexpectedToken(v)),
        };

        let first_entries = match lexer.next() {
            None | Some(TokenType::Slash) => Entries::new(),
            Some(TokenType::EnterSquareBracket) => Entries::parse_lexer(&mut lexer)?,
            Some(t) => return Err(ParseQueryError::UnexpectedToken(t)),
        };

        nodes.push(Node {
            ident: first_node,
            entries: first_entries,
        });
        loop {
            let node = match lexer.next() {
                Some(TokenType::Star) => NodeIdentifier::Any,
                Some(TokenType::DoublePoint) => NodeIdentifier::Parent,
                Some(t @ TokenType::String(_)) => NodeIdentifier::Named(parse_string(t)?),
                Some(TokenType::Alphanumeric(s)) => NodeIdentifier::Named(s),
                Some(t) => return Err(ParseQueryError::UnexpectedToken(t)),
                None => break,
            };
            let entries = match lexer.next() {
                None | Some(TokenType::Slash) => Entries::new(),
                Some(TokenType::EnterSquareBracket) => Entries::parse_lexer(&mut lexer)?,
                Some(t) => return Err(ParseQueryError::UnexpectedToken(t)),
            };
            nodes.push(Node {
                ident: node,
                entries,
            });
        }
        Ok(Path { nodes })
    }
}

fn parse_string<'a>(t: TokenType<'a>) -> Result<'a, &'a str> {
    let TokenType::String(s) = t else {
        return Err(ParseQueryError::UnexpectedToken(t));
    };
    match s.chars().next() {
        None => return Err(ParseQueryError::MalformedString(t)),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(t)),
        Some(_) => (),
    }

    match s.chars().last() {
        None => unreachable!(),
        Some(c) if c != '"' => return Err(ParseQueryError::MalformedString(t)),
        Some(_) => (),
    }
    // TODO: Convert escaped characters, output a Cow string
    return Ok(&s[1..s.len() - 1]);
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{Lexer, TokenType},
        parser::{parse_string, Entries, Node, NodeIdentifier, ParseQueryError, Path},
        util::hashmap,
    };

    #[inline]
    fn node_ident_only<'a>(ident: NodeIdentifier<'a>) -> Node<'a> {
        Node {
            ident,
            entries: Entries::new(),
        }
    }

    #[test]
    fn parser_strings() {
        fn test_string(input: &str, output: Result<&str, ParseQueryError>) {
            let mut lexer = Lexer::from(input);
            let token = lexer.next();
            assert_eq!(token, Some(TokenType::String(input)));
            assert_eq!(parse_string(token.unwrap()), output);
        }
        test_string("\"hello\"", Ok("hello"));
        test_string("\"hello world\"", Ok("hello world"));
        test_string(
            "\"hello world",
            Err(ParseQueryError::MalformedString(TokenType::String(
                "\"hello world",
            ))),
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
    fn path_named() {
        assert_eq!(
            Path::parse("node_name"),
            Ok(Path {
                nodes: vec![node_ident_only(NodeIdentifier::Named("node_name"))]
            })
        );
        assert_eq!(
            Path::parse("node1/node2"),
            Ok(Path {
                nodes: vec![
                    node_ident_only(NodeIdentifier::Named("node1")),
                    node_ident_only(NodeIdentifier::Named("node2"))
                ]
            })
        );
        assert_eq!(
            Path::parse(r#""node 1"/"node 2""#),
            Ok(Path {
                nodes: vec![
                    node_ident_only(NodeIdentifier::Named("node 1")),
                    node_ident_only(NodeIdentifier::Named("node 2"))
                ]
            })
        );
    }
}
