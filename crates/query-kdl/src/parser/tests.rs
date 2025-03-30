use std::borrow::Cow;

use crate::{
    lexer::{Lexer, TokenType},
    parser::{
        entries::EntryKind, error::ParseStringError, string, Entries, ParseError, Path, Selector,
        Value,
    },
    util::hashmap,
};

use super::SelectorKind;
#[test]
fn parser_strings() {
    fn test_string(input: &str, output: Result<&str, ParseError>) {
        let mut lexer = Lexer::from(input);
        let token = lexer.next();
        assert_eq!(token, Some(TokenType::String(input)));
        assert_eq!(
            string::parse_string(input).map_err(|e| e.into_parse_error(input)),
            output.map(Cow::Borrowed)
        );
    }
    test_string("\"hello\"", Ok("hello"));
    test_string("\"hello world\"", Ok("hello world"));
    test_string(
        "\"hello world",
        Err(ParseError::MalformedString(
            "\"hello world",
            ParseStringError::MissingEndOfString,
        )),
    );
}
#[test]
fn entries() {
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1 2 3"#)),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 0,
                value: Value::Integer(1)
            },
            EntryKind::Argument {
                position: 1,
                value: Value::Integer(2)
            },
            EntryKind::Argument {
                position: 2,
                value: Value::Integer(3)
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1 abc 3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 0,
                value: Value::Integer(1)
            },
            EntryKind::Argument {
                position: 1,
                value: Value::Str(Cow::Borrowed("abc"))
            },
            EntryKind::Argument {
                position: 2,
                value: Value::FloatingPoing(3.14)
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"a=b c=d"#)),
        Ok(Entries::from(vec![
            EntryKind::Property {
                name: Cow::Borrowed("a"),
                value: Value::Str(Cow::Borrowed("b"))
            },
            EntryKind::Property {
                name: Cow::Borrowed("c"),
                value: Value::Str(Cow::Borrowed("d"))
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name1=123 name2=abc name3=3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Property {
                name: Cow::Borrowed("name1"),
                value: Value::Integer(123)
            },
            EntryKind::Property {
                name: Cow::Borrowed("name2"),
                value: Value::Str(Cow::Borrowed("abc"))
            },
            EntryKind::Property {
                name: Cow::Borrowed("name3"),
                value: Value::FloatingPoing(3.14)
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1=123 2=abc 3=3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 1,
                value: Value::Integer(123)
            },
            EntryKind::Argument {
                position: 2,
                value: Value::Str(Cow::Borrowed("abc"))
            },
            EntryKind::Argument {
                position: 3,
                value: Value::FloatingPoing(3.14)
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(
            r#"1 2 3 a=4 b = 5 "p r o p"="v a l u e" 6 10=7"#
        )),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 0,
                value: Value::Integer(1)
            },
            EntryKind::Argument {
                position: 1,
                value: Value::Integer(2)
            },
            EntryKind::Argument {
                position: 2,
                value: Value::Integer(3)
            },
            EntryKind::Property {
                name: Cow::Borrowed("a"),
                value: Value::Integer(4)
            },
            EntryKind::Property {
                name: Cow::Borrowed("b"),
                value: Value::Integer(5)
            },
            EntryKind::Property {
                name: Cow::Borrowed("p r o p"),
                value: Value::Str(Cow::Borrowed("v a l u e"))
            },
            EntryKind::Argument {
                position: 3,
                value: Value::Integer(6)
            },
            EntryKind::Argument {
                position: 10,
                value: Value::Integer(7)
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1="#)),
        Err(ParseError::MissingEntryValue)
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name="#)),
        Err(ParseError::MissingEntryValue)
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"3.1=abc"#)),
        Err(ParseError::UnexpectedEntryIdentifier(Value::FloatingPoing(
            3.1
        )))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"=abc"#)),
        Err(ParseError::MissingEntryIdentifier)
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1=abc =cba"#)),
        Err(ParseError::MissingEntryIdentifier)
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name=abc =cba"#)),
        Err(ParseError::MissingEntryIdentifier)
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name=abc [ ]"#)),
        Err(ParseError::UnexpectedToken(TokenType::EnterSquareBracket))
    );
}

// fn node_only(node: SelectorKind) -> Selector<'a> {

// }

#[test]
fn path_named_one() {
    assert_eq!(
        Path::parse("node_name"),
        Ok(Path {
            nodes: vec![Selector::from(SelectorKind::Named(Cow::Borrowed(
                "node_name"
            )))]
        })
    );
}

#[test]
fn path_named_multi() {
    assert_eq!(
        Path::parse("node1/node2"),
        Ok(Path {
            nodes: vec![
                Selector::from(SelectorKind::Named(Cow::Borrowed("node1"))),
                Selector::from(SelectorKind::Named(Cow::Borrowed("node2"))),
            ]
        })
    );
}

#[test]
fn path_named_strings() {
    assert_eq!(
        Path::parse(r#""node 1"/"node 2""#),
        Ok(Path {
            nodes: vec![
                Selector::from(SelectorKind::Named(Cow::Borrowed("node 1"))),
                Selector::from(SelectorKind::Named(Cow::Borrowed("node 2"))),
            ]
        })
    );
}
// #[test]
// fn path_ident_root() {
//     assert_eq!(
//         Path::parse(r#"/node1"#),
//         Ok(Path {
//             nodes: vec![Selector::from(SelectorKind::Root(, Selector::from(SelectorKind::Named((Cow::Borrowed("node1"))]
//         })
//     );
//     assert_eq!(
//         Path::parse(r#"/node1/node2"#),
//         Ok(Path {
//             nodes: vec![
//                 Selector::from(SelectorKind::Root(,
//                 Selector::from(SelectorKind::Named((Cow::Borrowed("node1")),
//                 Selector::from(SelectorKind::Named((Cow::Borrowed("node2"))
//             ]
//         })
//     );
// }

#[test]
fn path_ident_anywhere() {
    assert_eq!(
        Path::parse(r#"**/node1"#),
        Ok(Path {
            nodes: vec![
                Selector::from(SelectorKind::Anywhere),
                Selector::from(SelectorKind::Named(Cow::Borrowed("node1")))
            ]
        })
    );
}

#[test]
fn path_ident_any() {
    assert_eq!(
        Path::parse(r#"*/node1"#),
        Ok(Path {
            nodes: vec![
                Selector::from(SelectorKind::Any),
                Selector::from(SelectorKind::Named(Cow::Borrowed("node1")))
            ]
        })
    );
}
#[test]
fn path_ident_parents() {
    assert_eq!(
        Path::parse(r#"../node1"#),
        Ok(Path {
            nodes: vec![
                Selector::from(SelectorKind::Parent),
                Selector::from(SelectorKind::Named(Cow::Borrowed("node1")))
            ]
        })
    );
}

#[test]
fn nodes_with_entries() {
    assert_eq!(
        Path::parse(r#"..[1]/node1[2]/*[3]/**[4]/{1..2}[5]"#),
        Ok(Path {
            nodes: vec![
                Selector {
                    node: SelectorKind::Parent,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Value::Integer(1)
                    },]))
                },
                Selector {
                    node: SelectorKind::Named(Cow::Borrowed("node1")),
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Value::Integer(2)
                    },]))
                },
                Selector {
                    node: SelectorKind::Any,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Value::Integer(3)
                    },]))
                },
                Selector {
                    node: SelectorKind::Anywhere,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Value::Integer(4)
                    },]))
                },
                Selector {
                    node: SelectorKind::Ranged(crate::parser::Range::Both(1, 2)),
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Value::Integer(5)
                    },]))
                },
            ]
        })
    );
}
#[test]
fn alphanum() {
    use {ParseStringError::*, Value::*};
    const PARSE: fn(&str) -> Result<Value<'_>, ParseStringError> = string::parse_alphanumeric;
    assert_eq!(PARSE("123"), Ok(Integer(123)));
    assert_eq!(PARSE("-123"), Ok(Integer(-123)));
    assert_eq!(PARSE("0x123"), Ok(Integer(0x123)));
    assert_eq!(PARSE("0b101"), Ok(Integer(0b101)));
    assert_eq!(PARSE("0o123"), Ok(Integer(0o123)));
    assert_eq!(PARSE("-0x123"), Ok(Integer(-0x123)));
    assert_eq!(PARSE("-0b101"), Ok(Integer(-0b101)));
    assert_eq!(PARSE("-0o123"), Ok(Integer(-0o123)));
    assert_eq!(PARSE("1.23"), Ok(FloatingPoing(1.23)));
    assert_eq!(PARSE("-1.23"), Ok(FloatingPoing(-1.23)));
    assert_eq!(PARSE("1.2.3"), Err(MalformedNumber));
    assert_eq!(PARSE("-1.2.3"), Err(MalformedNumber));
    assert_eq!(PARSE("1c0"), Err(MalformedNumber));
    assert_eq!(PARSE("-1c0"), Err(MalformedNumber));
    assert_eq!(PARSE("abc"), Ok(Str(Cow::Borrowed("abc"))));
    assert_eq!(PARSE("-abc"), Ok(Str(Cow::Borrowed("-abc"))));
    assert_eq!(PARSE("a1c"), Ok(Str(Cow::Borrowed("a1c"))));
    assert_eq!(PARSE("-a1c"), Ok(Str(Cow::Borrowed("-a1c"))));
}
#[test]
fn strings() {
    use {ParseStringError::*, Value::*};
    const PARSE: fn(&str) -> Result<Cow<'_, str>, ParseStringError> = string::parse_string;
    assert_eq!(PARSE(r#""""#), Ok(Cow::Borrowed("")));
    assert_eq!(PARSE(r#""abc""#), Ok(Cow::Borrowed("abc")));
    assert_eq!(
        PARSE(r#""a\nb\tc\rd\\e""#),
        Ok(Cow::Owned(String::from("a\nb\tc\rd\\e")))
    );
    assert_eq!(
        PARSE(r#""aa\x41\x5Abb""#),
        Ok(Cow::Owned(String::from("aaAZbb")))
    );
    assert_eq!(
        PARSE(r#""aa\u{4F60}\u{597D}\u{4E16}\u{754C}bb""#),
        Ok(Cow::Owned(String::from("aa你好世界bb")))
    );
    assert_eq!(PARSE(r#""aa\bbb""#), Err(UnknownEscape('b')));
    assert_eq!(PARSE(r#""aa\x89bb""#), Err(AsciiNotValid(0x89)));
    assert_eq!(PARSE(r#""aa\xTRbb""#), Err(NotHexDigit));
    assert_eq!(PARSE(r#""aa\u{DE01}bb""#), Err(UnicodeNotValid(0xDE01))); // Note: This character doesn't exists in the Unicode chart
}

#[test]
fn ranges() {
    use super::Range;
    use SelectorKind::Ranged;
    let parse = |s| Path::parse(s).map(|v| v.nodes);
    assert_eq!(
        parse("{1}"),
        Ok(vec![Selector::from(Ranged(Range::One(1)))])
    );
    assert_eq!(
        parse("{..2}"),
        Ok(vec![Selector::from(Ranged(Range::To(2)))])
    );
    assert_eq!(
        parse("{1..}"),
        Ok(vec![Selector::from(Ranged(Range::From(1)))])
    );
    assert_eq!(
        parse("{1..2}"),
        Ok(vec![Selector::from(Ranged(Range::Both(1, 2)))])
    );
    assert_eq!(parse("{..}"), Ok(vec![Selector::from(Ranged(Range::All))]));
    assert_eq!(
        parse("{abc..}"),
        Err(ParseError::RangeExpectingInteger(Value::Str(
            Cow::Borrowed("abc")
        )))
    );
    assert_eq!(
        parse("{..abc}"),
        Err(ParseError::RangeExpectingInteger(Value::Str(
            Cow::Borrowed("abc")
        )))
    );
    assert_eq!(
        parse("{1.2..}"),
        Err(ParseError::RangeExpectingInteger(Value::FloatingPoing(1.2)))
    );
    assert_eq!(
        parse("{1..2..}"),
        Err(ParseError::UnexpectedToken(TokenType::DoublePoint))
    );
    assert_eq!(
        parse("{..2..3}"),
        Err(ParseError::UnexpectedToken(TokenType::DoublePoint))
    );
    assert_eq!(parse("{1 2}"), Err(ParseError::RangeMissingSeparator));
    assert_eq!(parse("{}"), Err(ParseError::RangeEmpty));
}
