mod entries;
mod error;
mod string;
#[cfg(test)]
mod tests;
mod value;

use crate::lexer::{Lexer, TokenType};
pub use entries::{Entries, EntryKind};
pub use error::{ParseError, Result};
use std::{borrow::Cow, fmt::Display};
pub use value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Node<'a> {
    pub node: NodeKind<'a>,
    pub entries: Option<Entries<'a>>,
    pub range: Option<Range>,
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node)?;
        if let Some(entries) = &self.entries {
            write!(f, "{entries}")?;
        }
        if let Some(range) = &self.range {
            write!(f, "{range}")?;
        }
        Ok(())
    }
}

impl<'a> From<NodeKind<'a>> for Node<'a> {
    fn from(node: NodeKind<'a>) -> Self {
        Self {
            node,
            entries: None,
            range: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum NodeKind<'a> {
    /// "<name>" Node with a name
    Named(Cow<'a, str>),
    /// "*" Any nodes in the current scope
    Any,
    /// "**" Nodes starting anywhere in the doc
    Anywhere,
    /// ".." Parent node
    Parent,
}

impl<'a> Display for NodeKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Named(s) => write!(f, "{}", s),
            Self::Any => write!(f, "*"),
            Self::Anywhere => write!(f, "**"),
            Self::Parent => write!(f, ".."),
            _ => todo!(),
        }
    }
}

pub trait RangedIterator
where
    Self: Iterator + Sized,
{
    fn ranged(self, range: Option<&Range>) -> impl Iterator<Item = <Self as Iterator>::Item> {
        let Some(range) = range else {
            return self.skip(0).take(usize::MAX);
        };
        match range {
            Range::One(index) => self.skip(*index as _).take(1),
            Range::From(from) => self.skip(*from as _).take(usize::MAX),
            Range::To(to) => self.skip(0).take(*to as _),
            Range::Both(from, to) => self.skip(*from as _).take((*to - *from) as _),
            Range::All => self.skip(0).take(usize::MAX),
        }
    }
}
impl<I> RangedIterator for I where I: Iterator {}

#[derive(Clone, PartialEq, Debug)]
pub enum Range {
    /// {i}
    One(i128),
    /// {i..}
    From(i128),
    /// {..j}
    To(i128),
    /// {i..j}
    Both(i128, i128),
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

type Selectors<'a> = Vec<Node<'a>>;
#[derive(Clone, PartialEq, Debug)]
pub struct Path<'a> {
    nodes: Selectors<'a>,
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            write!(f, "{node}/")?;
        }
        Ok(())
    }
}

struct NodeBuilder<'a>(Option<Node<'a>>);

impl<'a> NodeBuilder<'a> {
    fn new() -> Self {
        Self(None)
    }
    fn set_node(&mut self, node: NodeKind<'a>) -> Result<'a, ()> {
        if self.0.is_some() {
            return Err(ParseError::NodeAlreadyDefined);
        }
        let _ = self.0.insert(Node::from(node));
        Ok(())
    }
    fn set_entries(&mut self, entries: Entries<'a>) -> Result<'a, ()> {
        let Some(node) = self.0.as_mut() else {
            return Err(ParseError::MissingNode);
        };
        if node.entries.is_some() {
            return Err(ParseError::EntriesAlreadyDefined);
        }
        let _ = node.entries.insert(entries);
        Ok(())
    }
    fn set_range(&mut self, range: Range) -> Result<'a, ()> {
        let Some(node) = self.0.as_mut() else {
            return Err(ParseError::MissingNode);
        };
        if node.range.is_some() {
            return Err(ParseError::RangeAlreadyDefined);
        }
        let _ = node.range.insert(range);
        Ok(())
    }
    fn pop(&mut self) -> Result<'a, Node<'a>> {
        self.0.take().ok_or_else(|| todo!("error missing node"))
    }
}

impl<'a> Path<'a> {
    pub fn parse(input: &'a str) -> Result<'a, Self> {
        let mut lexer = Lexer::from(input).peekable();
        let mut nodes = Vec::new();
        let mut node_builder = NodeBuilder::new();
        loop {
            let Some(token) = lexer.next() else {
                break;
            };
            match token {
                TokenType::Slash => nodes.push(node_builder.pop()?),
                TokenType::Star => node_builder.set_node(NodeKind::Any)?,
                TokenType::DoubleStar => node_builder.set_node(NodeKind::Anywhere)?,
                TokenType::DoublePoint => node_builder.set_node(NodeKind::Parent)?,
                TokenType::String(s) => node_builder.set_node(NodeKind::Named(
                    string::parse_string(s).map_err(|e| e.into_parse_error(s))?,
                ))?,
                TokenType::Alphanumeric(s) => {
                    let value = string::parse_alphanumeric(s).map_err(|e| e.into_parse_error(s))?;
                    let Value::String(name) = value else {
                        return Err(ParseError::NotANode);
                    };
                    node_builder.set_node(NodeKind::Named(name))?
                }
                TokenType::EnterSquareBracket => {
                    node_builder.set_entries(Entries::parse_lexer(&mut lexer)?)?
                }
                TokenType::EnterCurlyBracket => {
                    node_builder.set_range(Self::parse_range(&mut lexer)?)?
                }
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        if let Some(node) = node_builder.0 {
            nodes.push(node);
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
    #[inline]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}
