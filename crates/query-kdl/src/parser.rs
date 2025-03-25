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
    Ranged(i64, Option<i64>),
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
                    Selector::Entries(Entries::parse_lexer(&mut lexer)?)
                }
                TokenType::EnterCurlyBracket => Self::parse_range(&mut lexer)?,
                _ => return Err(ParseError::UnexpectedToken(token)),
            };
            nodes.push(selector);
        }

        return Ok(Self { nodes });
    }
    fn parse_range(lexer: &mut impl Iterator<Item = TokenType<'a>>) -> Result<'a, Selector<'a>> {
        let mut indices = [None, None];
        let mut has_sep = false;
        loop {
            let Some(token) = lexer.next() else {
                todo!("error missing end of range");
            };
            match token {
                TokenType::Alphanumeric(s) => {
                    let Value::Integer(index) =
                        string::parse_alphanumeric(s).map_err(|e| e.into_parse_error(s))?
                    else {
                        todo!("error not a number");
                    };
                    if indices[0].is_some() {
                        if !has_sep || indices[1].is_some() {
                            return Err(ParseError::UnexpectedToken(token));
                        }
                        indices[1] = Some(index);
                    } else {
                        indices[0] = Some(index);
                    }
                }
                TokenType::DoublePoint => {
                    if indices[1].is_some() {
                        return Err(ParseError::UnexpectedToken(token));
                    }
                    if indices[0].is_none() {
                        indices[0] = Some(0);
                    }
                    has_sep = true;
                }
                TokenType::LeaveCurlyBracket => break,
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        Ok(Selector::Ranged(
            indices[0].ok_or_else(|| todo!("error missing value"))?,
            indices[1],
        ))
    }
}
