use std::borrow::Cow;

use crate::{
    lexer::{Lexer, TokenType},
    parser::{
        entries::EntryKind, error::ParseStringError, string, Entries, Node, ParseError, Path, Value,
    },
    util::hashmap,
};

use super::NodeKind;
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
                value: Some(Value::Integer(1))
            },
            EntryKind::Argument {
                position: 1,
                value: Some(Value::Integer(2))
            },
            EntryKind::Argument {
                position: 2,
                value: Some(Value::Integer(3))
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1 abc 3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 0,
                value: Some(Value::Integer(1))
            },
            EntryKind::Argument {
                position: 1,
                value: Some(Value::String(Cow::Borrowed("abc")))
            },
            EntryKind::Argument {
                position: 2,
                value: Some(Value::FloatingPoing(3.14))
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"a=b c=d"#)),
        Ok(Entries::from(vec![
            EntryKind::Property {
                name: Cow::Borrowed("a"),
                value: Some(Value::String(Cow::Borrowed("b")))
            },
            EntryKind::Property {
                name: Cow::Borrowed("c"),
                value: Some(Value::String(Cow::Borrowed("d")))
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name1=123 name2=abc name3=3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Property {
                name: Cow::Borrowed("name1"),
                value: Some(Value::Integer(123))
            },
            EntryKind::Property {
                name: Cow::Borrowed("name2"),
                value: Some(Value::String(Cow::Borrowed("abc")))
            },
            EntryKind::Property {
                name: Cow::Borrowed("name3"),
                value: Some(Value::FloatingPoing(3.14))
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"1=123 2=abc 3=3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Argument {
                position: 1,
                value: Some(Value::Integer(123))
            },
            EntryKind::Argument {
                position: 2,
                value: Some(Value::String(Cow::Borrowed("abc")))
            },
            EntryKind::Argument {
                position: 3,
                value: Some(Value::FloatingPoing(3.14))
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
                value: Some(Value::Integer(1))
            },
            EntryKind::Argument {
                position: 1,
                value: Some(Value::Integer(2))
            },
            EntryKind::Argument {
                position: 2,
                value: Some(Value::Integer(3))
            },
            EntryKind::Property {
                name: Cow::Borrowed("a"),
                value: Some(Value::Integer(4))
            },
            EntryKind::Property {
                name: Cow::Borrowed("b"),
                value: Some(Value::Integer(5))
            },
            EntryKind::Property {
                name: Cow::Borrowed("p r o p"),
                value: Some(Value::String(Cow::Borrowed("v a l u e")))
            },
            EntryKind::Argument {
                position: 3,
                value: Some(Value::Integer(6))
            },
            EntryKind::Argument {
                position: 10,
                value: Some(Value::Integer(7))
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
            nodes: vec![Node::from(NodeKind::Named(Cow::Borrowed("node_name")))]
        })
    );
}

#[test]
fn path_named_multi() {
    assert_eq!(
        Path::parse("node1/node2"),
        Ok(Path {
            nodes: vec![
                Node::from(NodeKind::Named(Cow::Borrowed("node1"))),
                Node::from(NodeKind::Named(Cow::Borrowed("node2"))),
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
                Node::from(NodeKind::Named(Cow::Borrowed("node 1"))),
                Node::from(NodeKind::Named(Cow::Borrowed("node 2"))),
            ]
        })
    );
}
// #[test]
// fn path_ident_root() {
//     assert_eq!(
//         Path::parse(r#"/node1"#),
//         Ok(Path {
//             nodes: vec![Node::from(NodeKind::Root(, Node::from(NodeKind::Named((Cow::Borrowed("node1"))]
//         })
//     );
//     assert_eq!(
//         Path::parse(r#"/node1/node2"#),
//         Ok(Path {
//             nodes: vec![
//                 Node::from(NodeKind::Root(,
//                 Node::from(NodeKind::Named((Cow::Borrowed("node1")),
//                 Node::from(NodeKind::Named((Cow::Borrowed("node2"))
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
                Node::from(NodeKind::Anywhere),
                Node::from(NodeKind::Named(Cow::Borrowed("node1")))
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
                Node::from(NodeKind::Any),
                Node::from(NodeKind::Named(Cow::Borrowed("node1")))
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
                Node::from(NodeKind::Parent),
                Node::from(NodeKind::Named(Cow::Borrowed("node1")))
            ]
        })
    );
}

#[test]
fn nodes_with_entries() {
    assert_eq!(
        Path::parse(r#"..[1]/node1[2]/*[3]/**[4]"#),
        Ok(Path {
            nodes: vec![
                Node {
                    node: NodeKind::Parent,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Some(Value::Integer(1))
                    },])),
                    range: None,
                },
                Node {
                    node: NodeKind::Named(Cow::Borrowed("node1")),
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Some(Value::Integer(2))
                    },])),
                    range: None,
                },
                Node {
                    node: NodeKind::Any,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Some(Value::Integer(3))
                    },])),
                    range: None,
                },
                Node {
                    node: NodeKind::Anywhere,
                    entries: Some(Entries::from(vec![EntryKind::Argument {
                        position: 0,
                        value: Some(Value::Integer(4))
                    },])),
                    range: None,
                },
            ]
        })
    );

    assert_eq!(
        Path::parse("node1[1][2]"),
        Err(ParseError::EntriesAlreadyDefined)
    );
    assert_eq!(
        Path::parse("node1 node2"),
        Err(ParseError::NodeAlreadyDefined)
    );
    assert_eq!(
        Path::parse("node1 node2 [1]"),
        Err(ParseError::NodeAlreadyDefined)
    );
    assert_eq!(
        Path::parse("node1 [1] node2"),
        Err(ParseError::NodeAlreadyDefined)
    );
    assert_eq!(Path::parse("node1/[1]"), Err(ParseError::MissingNode));
}
#[test]
fn alphanum() {
    use string::parse_alphanumeric as parse;
    use {ParseStringError::*, Value::*};
    assert_eq!(parse("123"), Ok(Integer(123)));
    assert_eq!(parse("-123"), Ok(Integer(-123)));
    assert_eq!(parse("0x123"), Ok(Integer(0x123)));
    assert_eq!(parse("0b101"), Ok(Integer(0b101)));
    assert_eq!(parse("0o123"), Ok(Integer(0o123)));
    assert_eq!(parse("-0x123"), Ok(Integer(-0x123)));
    assert_eq!(parse("-0b101"), Ok(Integer(-0b101)));
    assert_eq!(parse("-0o123"), Ok(Integer(-0o123)));
    assert_eq!(parse("1.23"), Ok(FloatingPoing(1.23)));
    assert_eq!(parse("-1.23"), Ok(FloatingPoing(-1.23)));
    assert_eq!(parse("1.2.3"), Err(MalformedNumber));
    assert_eq!(parse("-1.2.3"), Err(MalformedNumber));
    assert_eq!(parse("1c0"), Err(MalformedNumber));
    assert_eq!(parse("-1c0"), Err(MalformedNumber));
    assert_eq!(parse("abc"), Ok(String(Cow::Borrowed("abc"))));
    assert_eq!(parse("-abc"), Ok(String(Cow::Borrowed("-abc"))));
    assert_eq!(parse("a1c"), Ok(String(Cow::Borrowed("a1c"))));
    assert_eq!(parse("-a1c"), Ok(String(Cow::Borrowed("-a1c"))));
}
#[test]
fn strings() {
    use string::parse_string as parse;
    use ParseStringError::*;
    assert_eq!(parse(r#""""#), Ok(Cow::Borrowed("")));
    assert_eq!(parse(r#""abc""#), Ok(Cow::Borrowed("abc")));
    assert_eq!(
        parse(r#""a\nb\tc\rd\\e""#),
        Ok(Cow::Owned(String::from("a\nb\tc\rd\\e")))
    );
    assert_eq!(
        parse(r#""aa\x41\x5Abb""#),
        Ok(Cow::Owned(String::from("aaAZbb")))
    );
    assert_eq!(
        parse(r#""aa\u{4F60}\u{597D}\u{4E16}\u{754C}bb""#),
        Ok(Cow::Owned(String::from("aa你好世界bb")))
    );
    assert_eq!(parse(r#""aa\bbb""#), Err(UnknownEscape('b')));
    assert_eq!(parse(r#""aa\x89bb""#), Err(AsciiNotValid(0x89)));
    assert_eq!(parse(r#""aa\xTRbb""#), Err(NotHexDigit));
    assert_eq!(parse(r#""aa\u{DE01}bb""#), Err(UnicodeNotValid(0xDE01))); // Note: This character doesn't exists in the Unicode chart
}

#[test]
fn ranges() {
    use super::Range;
    let make_node = |range| Node {
        node: NodeKind::Any,
        entries: None,
        range: Some(range),
    };
    let parse = |s| Path::parse(s).map(|v| v.nodes);
    assert_eq!(parse("*{1}"), Ok(vec![make_node(Range::One(1))]));
    assert_eq!(parse("*{..2}"), Ok(vec![make_node(Range::To(2))]));
    assert_eq!(parse("*{1..}"), Ok(vec![make_node(Range::From(1))]));
    assert_eq!(parse("*{1..2}"), Ok(vec![make_node(Range::Both(1, 2))]));
    assert_eq!(parse("*{..}"), Ok(vec![make_node(Range::All)]));
    assert_eq!(
        parse("*{abc..}"),
        Err(ParseError::RangeExpectingInteger(Value::String(
            Cow::Borrowed("abc")
        )))
    );
    assert_eq!(
        parse("*{..abc}"),
        Err(ParseError::RangeExpectingInteger(Value::String(
            Cow::Borrowed("abc")
        )))
    );
    assert_eq!(
        parse("*{1.2..}"),
        Err(ParseError::RangeExpectingInteger(Value::FloatingPoing(1.2)))
    );
    assert_eq!(
        parse("*{1..2..}"),
        Err(ParseError::UnexpectedToken(TokenType::DoublePoint))
    );
    assert_eq!(
        parse("*{..2..3}"),
        Err(ParseError::UnexpectedToken(TokenType::DoublePoint))
    );
    assert_eq!(parse("*{1 2}"), Err(ParseError::RangeMissingSeparator));
    assert_eq!(parse("*{}"), Err(ParseError::RangeEmpty));
}
