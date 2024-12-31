use super::FromMorePlugin;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::NarratableReportHandler;
use nu_plugin::EvaluatedCall;
use nu_plugin::{EngineInterface, PluginCommand, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, SyntaxShape, Type, Value as NuValue};
use std::collections::HashMap;

pub struct QueryKdl;

impl SimplePluginCommand for QueryKdl {
    type Plugin = FromMorePlugin;

    fn name(&self) -> &str {
        "query kdl"
    }

    fn description(&self) -> &str {
        "select kdl data"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::String, Type::String)
            .required("query", SyntaxShape::String, "xpath like query")
    }

    fn run(
        &self,
        _plugin: &FromMorePlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &NuValue,
    ) -> Result<NuValue, LabeledError> {
        let query = call.positional.get(0).ok_or(
            LabeledError::new("Missing argument").with_label("query is missing", call.head),
        )?;

        let NuValue::String { val: input, .. } = input else {
            return Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            );
        };
        let query_span = query.span();
        let NuValue::String { val: query_str, .. } = query else {
            return Err(LabeledError::new("Expected String for query").with_label(
                format!("Expected a string, got {}", query.get_type()),
                query_span,
            ));
        };
        let query = Path::parse(&query_str).map_err(|e| {
            LabeledError::new("Failed to parse query").with_label(e.to_string(), query_span)
        })?;

        let kdoc = match KdlDocument::parse(input) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new("Failed to parse KDL format")
                    .with_label(e.to_string(), call.head))
            }
        };
        todo!()
    }
}

impl QueryKdl {}

enum NodeIdentifier<'a> {
    /// Node with a name
    Named(&'a str),
    /// Any nodes in the current scope
    Any,
    /// Root node
    Root,
    /// Nodes starting anywhere in the doc
    Anywhere,
    /// Parent node
    Parent,
}

struct Node<'a> {
    ident: NodeIdentifier<'a>,
    arguments: Vec<&'a str>,
    properties: HashMap<&'a str, &'a str>,
}

struct Path<'a> {
    nodes: Vec<Node<'a>>,
}

#[derive(thiserror::Error, Debug)]
enum ParseQueryError {}

impl<'a> Path<'a> {
    fn parse(input: &'a str) -> Result<Self, ParseQueryError> {
        todo!()
    }
}
#[derive(Debug, PartialEq, Eq)]
enum TokenType<'a> {
    String(&'a str),
    Alphanumeric(&'a str),
    Slash,
    DoubleSlash,
    Point,
    DoublePoint,
    Star,
    EnterSquareBracket,
    LeaveSquareBracket,
    Equal,
    Pipe,
    Unknown(&'a str),
}

struct Lexer<'a> {
    input: &'a str,
}

impl<'a, T: AsRef<str> + ?Sized> From<&'a T> for Lexer<'a> {
    fn from(value: &'a T) -> Self {
        Self {
            input: value.as_ref(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenType<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespaces
        let byte_offset = self
            .input
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum();

        self.input = &self.input[byte_offset..];
        let c_token = self.input.chars().next();
        match c_token {
            None => None,
            Some('"' | '\'') => self.get_text(),
            Some(c) if c.is_alphanumeric() => self.get_alphanumeric(),
            Some(_) => self.get_token(),
        }
    }
}
impl<'a> Lexer<'a> {
    fn get_text(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut iter_chars = self.input.chars();
        let c_str = iter_chars.next().unwrap();
        let mut escaped = false;
        let mut len_str = c_str.len_utf8();
        for c in iter_chars {
            len_str += c.len_utf8();
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
            } else if c == c_str {
                break;
            }
        }
        self.advance_and_return(len_str).map(TokenType::String)
    }
    fn get_alphanumeric(&mut self) -> Option<<Self as Iterator>::Item> {
        let len = self
            .input
            .chars()
            .take_while(|c| c.is_alphanumeric())
            .map(char::len_utf8)
            .sum();
        self.advance_and_return(len).map(TokenType::Alphanumeric)
    }
    fn get_token(&mut self) -> Option<<Self as Iterator>::Item> {
        use TokenType::*;
        let mut iter_chars = self.input.chars().map(|c| (c.len_utf8(), c));
        let (mut offset, c) = iter_chars.next()?;
        let result = match c {
            '/' => match iter_chars.next() {
                Some((l, '/')) => {
                    offset += l;
                    DoubleSlash
                }
                Some(_) | None => Slash,
            },
            '[' => EnterSquareBracket,
            ']' => LeaveSquareBracket,
            '.' => match iter_chars.next() {
                Some((l, '.')) => {
                    offset += l;
                    DoublePoint
                }
                Some(_) | None => Point,
            },
            '*' => Star,
            '=' => Equal,
            '|' => Pipe,
            c => Unknown(&self.input[0..c.len_utf8()]),
        };
        self.input = &self.input[offset..];
        Some(result)
    }
    #[inline]
    fn advance_and_return(&mut self, offset: usize) -> Option<&'a str> {
        let result = &self.input[0..offset];
        self.input = &self.input[offset..];
        Some(result)
    }
}

#[cfg(test)]
mod lexer_tests {
    use super::{Lexer, TokenType};
    #[test]
    fn text() {
        let mut lexer = Lexer::from("\"hello\"");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello\"")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn spaced_text() {
        let mut lexer = Lexer::from("   \"hello\"  ");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello\"")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn wrong_text() {
        let mut lexer = Lexer::from("\"hello");
        assert_eq!(lexer.next(), Some(TokenType::String("\"hello")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn text_escaped() {
        let mut lexer = Lexer::from(r#""hello\"world""#);
        assert_eq!(lexer.next(), Some(TokenType::String(r#""hello\"world""#)));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_text() {
        let mut lexer = Lexer::from(r#""hello" "world""#);
        assert_eq!(lexer.next(), Some(TokenType::String(r#""hello""#)));
        assert_eq!(lexer.next(), Some(TokenType::String(r#""world""#)));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn alpha() {
        let mut lexer = Lexer::from("abc");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn alphanumeric() {
        let mut lexer = Lexer::from("abc123");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc123")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn numeric() {
        let mut lexer = Lexer::from("123");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn numalpha() {
        let mut lexer = Lexer::from("123abc");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123abc")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_alpha() {
        let mut lexer = Lexer::from("abc def ghijk");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("def")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("ghijk")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_numeric() {
        let mut lexer = Lexer::from("123 456 10938");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("123")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("456")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("10938")));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn multiple_alphanumeric() {
        let mut lexer = Lexer::from("abc 4476 ghijk 73ab35");
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("abc")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("4476")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("ghijk")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("73ab35")));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn token_sqbrackets() {
        let mut lexer = Lexer::from("[]");
        assert_eq!(lexer.next(), Some(TokenType::EnterSquareBracket));
        assert_eq!(lexer.next(), Some(TokenType::LeaveSquareBracket));
        assert_eq!(lexer.next(), None);
    }
    #[test]
    fn token_relatives() {
        let mut lexer = Lexer::from("/./..//*");
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::Point));
        assert_eq!(lexer.next(), Some(TokenType::Slash));
        assert_eq!(lexer.next(), Some(TokenType::DoublePoint));
        assert_eq!(lexer.next(), Some(TokenType::DoubleSlash));
        assert_eq!(lexer.next(), Some(TokenType::Star));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn entries() {
        let mut lexer = Lexer::from(r#"[name=value 1 "2" name1 = value1 | name = value1]"#);
        assert_eq!(lexer.next(), Some(TokenType::EnterSquareBracket));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value")));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("1")));
        assert_eq!(lexer.next(), Some(TokenType::String(r#""2""#)));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name1")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value1")));
        assert_eq!(lexer.next(), Some(TokenType::Pipe));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("name")));
        assert_eq!(lexer.next(), Some(TokenType::Equal));
        assert_eq!(lexer.next(), Some(TokenType::Alphanumeric("value1")));
        assert_eq!(lexer.next(), Some(TokenType::LeaveSquareBracket));
        assert_eq!(lexer.next(), None);
    }
}
