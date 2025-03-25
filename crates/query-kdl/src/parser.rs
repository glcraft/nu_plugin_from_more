mod entries;
mod error;
mod string;
#[cfg(test)]
mod tests;
mod value;

use crate::lexer::{Lexer, TokenType};
use entries::Entries;
pub use error::{ParseError, Result};
use std::{borrow::Cow, fmt::Display};
use value::Value;

#[derive(Clone, PartialEq, Debug)]
pub enum Selector<'a> {
    /// "<name>" Node with a name
    Named(Cow<'a, str>),
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
    Ranged(Range),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Range {
    /// {i}
    One(i64),
    /// {i..}
    From(i64),
    /// {..j}
    To(i64),
    /// {i..j}
    Both(i64, i64),
    /// {..}
    All,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::One(i) => write!(f, "{{{0}}}", i),
            Range::From(i) => write!(f, "{{{0}..}}", i),
            Range::To(i) => write!(f, "{{..{0}}}", i),
            Range::Both(i, j) => write!(f, "{{{0}..{1}}}", i, j),
            Range::All => write!(f, "{{..}}"),
        }
    }
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
            Self::Ranged(range) => range.fmt(f),
            _ => todo!(),
        }
    }
}

type Selectors<'a> = Vec<Selector<'a>>;
#[derive(Clone, PartialEq, Debug)]
pub struct Path<'a> {
    nodes: Selectors<'a>,
}
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
                        return Err(ParseError::UnexpectedToken(token));
                    }
                }
                TokenType::Star => Selector::Any,
                TokenType::DoublePoint => Selector::Parent,
                TokenType::String(s) => {
                    Selector::Named(string::parse_string(s).map_err(|e| e.into_parse_error(s))?)
                }
                TokenType::Alphanumeric(s) => {
                    let value = string::parse_alphanumeric(s).map_err(|e| e.into_parse_error(s))?;
                    let Value::Str(name) = value else {
                        return Err(ParseError::NotANode);
                    };
                    Selector::Named(name)
                }
                TokenType::EnterSquareBracket => {
                    Entries::parse_lexer(&mut lexer).map(Selector::Entries)?
                }
                TokenType::EnterCurlyBracket => {
                    Self::parse_range(&mut lexer).map(Selector::Ranged)?
                }
                _ => return Err(ParseError::UnexpectedToken(token)),
            };
            nodes.push(selector);
        }

        return Ok(Self { nodes });
    }
    fn parse_range(lexer: &mut impl Iterator<Item = TokenType<'a>>) -> Result<'a, Range> {
        let mut indices = [None, None];
        let mut has_sep = false;
        loop {
            let Some(token) = lexer.next() else {
                todo!("error missing end of range");
            };
            match token {
                TokenType::Alphanumeric(s) => {
                    let value = string::parse_alphanumeric(s).map_err(|e| e.into_parse_error(s))?;
                    let index = match value {
                        Value::Integer(i) => i,
                        t => return Err(ParseError::RangeExpectingInteger(t)),
                    };
                    let i = has_sep as usize;
                    if indices[i].is_some() {
                        return Err(ParseError::RangeMissingSeparator);
                    }
                    indices[i] = Some(index);
                }
                TokenType::DoublePoint => {
                    if has_sep {
                        return Err(ParseError::UnexpectedToken(TokenType::DoublePoint));
                    }
                    has_sep = true;
                }
                TokenType::LeaveCurlyBracket => break,
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        match (indices, has_sep) {
            ([None, None], false) => Err(ParseError::RangeEmpty),
            ([None, None], true) => Ok(Range::All),
            ([Some(i), None], false) => Ok(Range::One(i)),
            ([Some(i), None], true) => Ok(Range::From(i)),
            ([None, Some(i)], true) => Ok(Range::To(i)),
            ([None, Some(_)], false) => unreachable!(),

            ([Some(i), Some(j)], true) => Ok(Range::Both(i, j)),
            ([Some(_), Some(_)], false) => Err(ParseError::RangeMissingSeparator),
        }
    }
}
