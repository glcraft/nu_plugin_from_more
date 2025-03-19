mod entries;
mod error;
#[cfg(test)]
mod tests;
mod util;
mod value;

use crate::lexer::{Lexer, TokenType};
use entries::Entries;
pub use error::{ParseQueryError, Result};
use std::{collections::HashMap, fmt::Display};
use value::Value;

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
                        return Err(ParseQueryError::UnexpectedToken(token));
                    }
                }
                TokenType::Star => Selector::Any,
                TokenType::DoublePoint => Selector::Parent,
                TokenType::String(s) => Selector::Named(util::parse_string(s)?),
                TokenType::Alphanumeric(s) => {
                    let value = util::parse_alphanumeric(s)?;
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
}
