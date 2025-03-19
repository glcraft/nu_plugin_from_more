use std::fmt::Display;

#[derive(Clone, PartialEq, Debug)]
pub enum Value<'a> {
    /// String
    Str(&'a str),
    /// Integer
    Integer(i64),
    /// Floating point
    FloatingPoing(f64),
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{}", s),
            Self::Integer(i) => write!(f, "{}", i),
            Self::FloatingPoing(fp) => write!(f, "{}", fp),
        }
    }
}
