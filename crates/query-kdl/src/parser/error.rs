use super::Value;
use crate::lexer::TokenType;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseStringError {
    EmptyString,
    MissingBeginOfString,
    MissingEndOfString,
    UnknownEscape(char),
    NotHexDigit,
    NotAsciiCodepoint(u8),
    NotValidCodepoint(u32),
    UnicodeMoreThanSixDigits,
    UnicodeOutOfBound(u32),
    ExpectedCurlyBracket,
    MalformedNumber,
}

impl ParseStringError {
    #[inline]
    pub fn into_parse_error<'a>(self, origin: &'a str) -> ParseError<'a> {
        ParseError::MalformedString(origin, self)
    }
}

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum ParseError<'a> {
    #[error("unexpected token: {0}")]
    UnexpectedToken(TokenType<'a>),
    #[error("malformed string (missing \"): {0}")]
    MalformedString(&'a str, ParseStringError),
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

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;
