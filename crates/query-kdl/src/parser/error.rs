use super::Value;
use crate::lexer::TokenType;

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

pub type Result<'a, T> = std::result::Result<T, ParseQueryError<'a>>;
