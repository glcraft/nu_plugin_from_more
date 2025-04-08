use super::Value;
use crate::lexer::TokenType;

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum ParseStringError {
    #[error("The string is empty")]
    EmptyString,
    #[error("The string misses \" at the beginning")]
    MissingBeginOfString,
    #[error("The string misses \" at the end")]
    MissingEndOfString,
    #[error("This escape does not exists: \\{0}")]
    UnknownEscape(char),
    #[error("Expected hexadecimal number, but had something else")]
    NotHexDigit,
    #[error("Ascii escape code not valid: \\x{0:02X}")]
    AsciiNotValid(u8),
    #[error("Unicode escape code not valid: \\u{{{0:X}}}")]
    UnicodeNotValid(u32),
    #[error("Unicode escape code must have at most 6 digits")]
    UnicodeMoreThanSixDigits,
    #[error("Unicode escape code must be at most 0x10FFFF (found \\u{{{0}}})")]
    UnicodeOutOfBound(u32),
    #[error("A curly bracket is missing")]
    ExpectedCurlyBracket,
    #[error("Number detected as malformed")]
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
    #[error("The string \"{0}\" is malformed: {1}")]
    MalformedString(&'a str, ParseStringError),
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
    #[error("missing node before entries")]
    MissingNode,
    #[error("A node is already defined before")]
    NodeAlreadyDefined,
    #[error("The entries was already defined for this node")]
    EntriesAlreadyDefined,
    #[error("The range was already defined for this node")]
    RangeAlreadyDefined,
    #[error("expected an integer number, got: {0}")]
    RangeExpectingInteger(Value<'a>),
    #[error("The range is empty")]
    RangeEmpty,
    #[error("The range separator (between numbers) is missing")]
    RangeMissingSeparator,
}

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;
