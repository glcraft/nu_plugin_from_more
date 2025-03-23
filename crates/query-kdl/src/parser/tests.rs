use crate::{
    lexer::{Lexer, TokenType},
    parser::{entries::EntryKind, util, Entries, ParseError, Path, Selector, Value},
    util::hashmap,
};

#[test]
fn parser_strings() {
    fn test_string(input: &str, output: Result<&str, ParseError>) {
        let mut lexer = Lexer::from(input);
        let token = lexer.next();
        assert_eq!(token, Some(TokenType::String(input)));
        assert_eq!(util::parse_string(input), output);
    }
    test_string("\"hello\"", Ok("hello"));
    test_string("\"hello world\"", Ok("hello world"));
    test_string(
        "\"hello world",
        Err(ParseError::MalformedString("\"hello world")),
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
                value: Value::Str("abc")
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
                name: "a",
                value: Value::Str("b")
            },
            EntryKind::Property {
                name: "c",
                value: Value::Str("d")
            },
        ]))
    );
    assert_eq!(
        Entries::parse_lexer(&mut Lexer::from(r#"name1=123 name2=abc name3=3.14"#)),
        Ok(Entries::from(vec![
            EntryKind::Property {
                name: "name1",
                value: Value::Integer(123)
            },
            EntryKind::Property {
                name: "name2",
                value: Value::Str("abc")
            },
            EntryKind::Property {
                name: "name3",
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
                value: Value::Str("abc")
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
                name: "a",
                value: Value::Integer(4)
            },
            EntryKind::Property {
                name: "b",
                value: Value::Integer(5)
            },
            EntryKind::Property {
                name: "p r o p",
                value: Value::Str("v a l u e")
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
#[test]
fn path_named_one() {
    assert_eq!(
        Path::parse("node_name"),
        Ok(Path {
            nodes: vec![Selector::Named("node_name")]
        })
    );
}

#[test]
fn path_named_multi() {
    assert_eq!(
        Path::parse("node1/node2"),
        Ok(Path {
            nodes: vec![Selector::Named("node1"), Selector::Named("node2")]
        })
    );
}

#[test]
fn path_named_strings() {
    assert_eq!(
        Path::parse(r#""node 1"/"node 2""#),
        Ok(Path {
            nodes: vec![Selector::Named("node 1"), Selector::Named("node 2")]
        })
    );
}
#[test]
fn path_ident_root() {
    assert_eq!(
        Path::parse(r#"/node1"#),
        Ok(Path {
            nodes: vec![Selector::Root, Selector::Named("node1")]
        })
    );
    assert_eq!(
        Path::parse(r#"/node1/node2"#),
        Ok(Path {
            nodes: vec![
                Selector::Root,
                Selector::Named("node1"),
                Selector::Named("node2")
            ]
        })
    );
}

#[test]
fn path_ident_anywhere() {
    assert_eq!(
        Path::parse(r#"//node1"#),
        Ok(Path {
            nodes: vec![Selector::Anywhere, Selector::Named("node1")]
        })
    );
}

#[test]
fn path_ident_any() {
    assert_eq!(
        Path::parse(r#"/*/node1"#),
        Ok(Path {
            nodes: vec![Selector::Root, Selector::Any, Selector::Named("node1")]
        })
    );
}
#[test]
fn path_ident_parents() {
    assert_eq!(
        Path::parse(r#"/../node1"#),
        Ok(Path {
            nodes: vec![Selector::Root, Selector::Parent, Selector::Named("node1")]
        })
    );
}
#[test]
fn alphanum() {
    use {ParseError::*, Value::*};
    let parse = util::parse_alphanumeric;
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
    assert_eq!(parse("1.2.3"), Err(MalformedNumber("1.2.3")));
    assert_eq!(parse("-1.2.3"), Err(MalformedNumber("-1.2.3")));
    assert_eq!(parse("1c0"), Err(MalformedNumber("1c0")));
    assert_eq!(parse("-1c0"), Err(MalformedNumber("-1c0")));
    assert_eq!(parse("abc"), Ok(Str("abc")));
    assert_eq!(parse("-abc"), Ok(Str("-abc")));
    assert_eq!(parse("a1c"), Ok(Str("a1c")));
    assert_eq!(parse("-a1c"), Ok(Str("-a1c")));
}
