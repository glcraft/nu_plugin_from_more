use std::{borrow::Cow, fmt::Display};

#[derive(Clone, PartialEq, Debug)]
pub enum Value<'a> {
    /// String
    String(Cow<'a, str>),
    /// Integer
    Integer(i64),
    /// Floating point
    FloatingPoing(f64),
    /// Boolean
    Boolean(bool),
    /// Null
    Null,
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Integer(i) => write!(f, "{}", i),
            Self::FloatingPoing(fp) => write!(f, "{}", fp),
            Self::Boolean(b) => write!(f, "#{}", if *b { "true" } else { "false" }),
            Self::Null => write!(f, "#null"),
        }
    }
}
