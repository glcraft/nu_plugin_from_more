use super::{string, ParseError, Result, Value};
use crate::lexer::TokenType;
use std::{borrow::Cow, fmt::Display};

#[derive(Clone, PartialEq, Debug)]
pub enum EntryKind<'a> {
    Argument {
        position: u64,
        value: Option<Value<'a>>,
    },
    Property {
        name: Cow<'a, str>,
        value: Option<Value<'a>>,
    },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Entries<'a>(Vec<EntryKind<'a>>);

impl<'a> Display for Entries<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for (i, v) in self.0.iter().enumerate() {
            match v {
                EntryKind::Argument { position, value } if i != (*position as _) => write!(
                    f,
                    "{}={}",
                    position,
                    value.as_ref().unwrap_or(&Value::String(Cow::Borrowed("_")))
                ),
                EntryKind::Argument { value, .. } => write!(
                    f,
                    "{} ",
                    value.as_ref().unwrap_or(&Value::String(Cow::Borrowed("_")))
                ),
                EntryKind::Property { name, value } => write!(
                    f,
                    "{}={} ",
                    name,
                    value.as_ref().unwrap_or(&Value::String(Cow::Borrowed("_")))
                ), // TODO: quote string if not alphanumeric
            }?;
        }
        write!(f, "]")
    }
}
impl<'a> From<Vec<EntryKind<'a>>> for Entries<'a> {
    fn from(value: Vec<EntryKind<'a>>) -> Self {
        Entries(value)
    }
}

impl<'a> Entries<'a> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn parse_lexer(lexer: &mut impl Iterator<Item = TokenType<'a>>) -> Result<'a, Self> {
        let mut entries = vec![];
        let mut arg_pos = 0;
        let mut prop_name = None;
        let mut is_unnamed_arg = false;
        while let Some(token) = lexer.next() {
            let value = match token {
                TokenType::LeaveSquareBracket => break,
                TokenType::Alphanumeric(s) => {
                    is_unnamed_arg = true;
                    let v = string::parse_alphanumeric(s).map_err(|e| e.into_parse_error(s))?;
                    (!matches!(v, Value::String(ref s) if s == "_")).then(|| v)
                }
                TokenType::String(s) => {
                    is_unnamed_arg = true;
                    Some(Value::String(
                        string::parse_string(s).map_err(|e| e.into_parse_error(s))?,
                    ))
                }
                TokenType::Equal => {
                    if prop_name.is_some() {
                        return Err(ParseError::DoubleEqual);
                    }
                    let Some(EntryKind::Argument { position: _, value }) = entries.pop() else {
                        return Err(ParseError::MissingEntryIdentifier);
                    };
                    if !is_unnamed_arg {
                        return Err(ParseError::MissingEntryIdentifier);
                    }
                    prop_name = Some(value);
                    arg_pos -= 1;
                    is_unnamed_arg = false;
                    continue;
                }
                t => return Err(ParseError::UnexpectedToken(t)),
            };
            match prop_name {
                Some(None) => return Err(ParseError::UndefinedEntryIdentifier),
                Some(Some(Value::String(name))) => {
                    entries.push(EntryKind::Property { name, value });
                    prop_name = None;
                    is_unnamed_arg = false;
                }
                Some(Some(Value::Integer(position))) => {
                    entries.push(EntryKind::Argument {
                        position: position as _,
                        value,
                    });
                    prop_name = None;
                    is_unnamed_arg = false;
                }
                Some(Some(s @ _)) => return Err(ParseError::UnexpectedEntryIdentifier(s)),
                None => {
                    entries.push(EntryKind::Argument {
                        position: arg_pos,
                        value,
                    });
                    arg_pos += 1;
                }
            }
        }
        if prop_name.is_some() {
            return Err(ParseError::MissingEntryValue);
        }
        Ok(Entries(entries))
    }
    pub fn entries(&self) -> &[EntryKind] {
        &self.0
    }
}
impl<'a> Default for Entries<'a> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
