use kdl::{KdlDocument, KdlNode};
use std::{borrow::Cow, sync::LazyLock};

use crate::{
    lexer::Lexer,
    parser::{Entries, EntryKind, Path},
    resolve::Resolver,
};

static KDL_DOC: LazyLock<KdlDocument> = LazyLock::new(|| {
    r#"
        node1
        node2 1 2 3
        node2
        node3 a b c
        node3 0 2 0
        node_prop hello=world
        node_prop hello=world 123
        node_prop hello=world foo=bar
        node_children {
            node1 1
            node2 2
            node3 3
        }
        node_multiple {
            node 1
            node 2
            node 3
            node 4
            node 5
        }
        article {
            contents {
                section "First section" {
                    paragraph "This is the first paragraph"
                    paragraph "This is the second paragraph"
                }
            }
            contents {
                section "Second section" {
                    paragraph "This is the third paragraph"
                    paragraph "This is the forth paragraph"
                }
            }
        }
        "#
    .parse()
    .expect("kdl not well formatted")
});
#[derive(Debug)]
struct TestNode {
    name: &'static str,
    entries: Entries<'static>,
}
#[derive(Debug)]
struct TestNodes(Vec<TestNode>);

impl PartialEq<TestNode> for KdlNode {
    fn eq(&self, other: &TestNode) -> bool {
        let Some(name) = self.name().repr() else {
            return false;
        };
        name == other.name && *self.entries() == other.entries
    }
}

impl<'a> PartialEq<TestNodes> for Vec<&'a KdlNode> {
    fn eq(&self, other: &TestNodes) -> bool {
        if self.len() != other.0.len() {
            return false;
        }
        for (left, right) in self.iter().zip(other.0.iter()) {
            if **left != *right {
                return false;
            }
        }
        true
    }
}

fn entries(input: &'static str) -> Entries<'static> {
    let mut lexer = Lexer::from(input);
    Entries::parse_lexer(&mut lexer).expect("error while parsing entries in tests")
}

#[test]
fn query_named_node() {
    let query = Path::parse("node2").unwrap();
    let found = Resolver::resolve(&*KDL_DOC, query);
    assert_eq!(
        found,
        TestNodes(vec![
            TestNode {
                name: "node2",
                entries: entries("1 2 3")
            },
            TestNode {
                name: "node2",
                entries: Entries::default()
            }
        ])
    );
}
#[test]
fn query_any_node() {
    let query = Path::parse("node_children/*").unwrap();

    println!("{:#?}", query);
    let found = Resolver::resolve(&*KDL_DOC, query);
    assert_eq!(
        found,
        TestNodes(vec![
            TestNode {
                name: "node1",
                entries: entries("1")
            },
            TestNode {
                name: "node2",
                entries: entries("2")
            },
            TestNode {
                name: "node3",
                entries: entries("3")
            },
        ])
    );
}
#[test]
fn query_parent_node() {
    let query = Path::parse("node_children/node1/..").unwrap();
    let found = Resolver::resolve(&*KDL_DOC, query);
    assert_eq!(
        found,
        TestNodes(vec![TestNode {
            name: "node_children",
            entries: Entries::default()
        },])
    );
}
#[test]
fn query_parent_node_multi() {
    let query = Path::parse("node_children/*/..").unwrap();
    let found = Resolver::resolve(&*KDL_DOC, query);
    assert_eq!(
        found,
        TestNodes(vec![
            TestNode {
                name: "node_children",
                entries: Entries::default()
            },
            TestNode {
                name: "node_children",
                entries: Entries::default()
            },
            TestNode {
                name: "node_children",
                entries: Entries::default()
            }
        ])
    );
}
#[test]
fn query_ranges() {
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/*{1}").unwrap()),
        TestNodes(vec![TestNode {
            name: "node",
            entries: entries("2")
        },])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/..{0}").unwrap()),
        TestNodes(vec![TestNode {
            name: "node_multiple",
            entries: Entries::new()
        },])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node2/..{1}").unwrap()),
        TestNodes(vec![])
    );

    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/node{1}").unwrap()),
        TestNodes(vec![TestNode {
            name: "node",
            entries: entries("2")
        },])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/node{1..}").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node",
                entries: entries("2")
            },
            TestNode {
                name: "node",
                entries: entries("3")
            },
            TestNode {
                name: "node",
                entries: entries("4")
            },
            TestNode {
                name: "node",
                entries: entries("5")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/node{..3}").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node",
                entries: entries("1")
            },
            TestNode {
                name: "node",
                entries: entries("2")
            },
            TestNode {
                name: "node",
                entries: entries("3")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/node{1..3}").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node",
                entries: entries("2")
            },
            TestNode {
                name: "node",
                entries: entries("3")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("node_multiple/node{..}").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node",
                entries: entries("1")
            },
            TestNode {
                name: "node",
                entries: entries("2")
            },
            TestNode {
                name: "node",
                entries: entries("3")
            },
            TestNode {
                name: "node",
                entries: entries("4")
            },
            TestNode {
                name: "node",
                entries: entries("5")
            },
        ])
    );
}
#[test]
fn query_entries() {
    use crate::parser::Value;
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[_ 2]").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node2",
                entries: entries("1 2 3")
            },
            TestNode {
                name: "node3",
                entries: entries("0 2 0")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[1=2]").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node2",
                entries: entries("1 2 3")
            },
            TestNode {
                name: "node3",
                entries: entries("0 2 0")
            },
        ])
    );
    //Note : 3 arguments defined, no matter what are their values
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[_ _ _]").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node2",
                entries: entries("1 2 3")
            },
            TestNode {
                name: "node3",
                entries: entries("a b c")
            },
            TestNode {
                name: "node3",
                entries: entries("0 2 0")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[hello=world]").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "node_prop",
                entries: entries("hello=world")
            },
            TestNode {
                name: "node_prop",
                entries: entries("hello=world 123")
            },
            TestNode {
                name: "node_prop",
                entries: entries("hello=world foo=bar")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[hello=world foo=bar]").unwrap()),
        TestNodes(vec![TestNode {
            name: "node_prop",
            entries: entries("hello=world foo=bar")
        },])
    );
    // Just a test to see that the entries parser works well here
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[hello=world 123]").unwrap()),
        TestNodes(vec![TestNode {
            name: "node_prop",
            entries: Entries::from(vec![
                EntryKind::Property {
                    name: Cow::Borrowed("hello"),
                    value: Some(Value::String(Cow::Borrowed("world")))
                },
                EntryKind::Argument {
                    position: 0,
                    value: Some(Value::Integer(123))
                }
            ])
        },])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("*[hello=world foo=bar]").unwrap()),
        TestNodes(vec![TestNode {
            name: "node_prop",
            entries: entries("hello=world foo=bar")
        },])
    );
    assert_eq!(
        Resolver::resolve(
            &*KDL_DOC,
            Path::parse("article/contents/section/paragraph[\"This is the first paragraph\"]")
                .unwrap()
        ),
        TestNodes(vec![TestNode {
            name: "paragraph",
            entries: entries("\"This is the first paragraph\"")
        },])
    );
    assert_eq!(
        Resolver::resolve(
            &*KDL_DOC,
            Path::parse("*/*/*/*[\"This is the third paragraph\"]").unwrap()
        ),
        TestNodes(vec![TestNode {
            name: "paragraph",
            entries: entries("\"This is the third paragraph\"")
        },])
    );
    assert_eq!(
        Resolver::resolve(
            &*KDL_DOC,
            Path::parse("article/contents/section[\"Second section\"]/*").unwrap()
        ),
        TestNodes(vec![
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the third paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the forth paragraph\"")
            },
        ])
    );
}
#[test]
fn query_anywhere() {
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("article/**").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "contents",
                entries: Entries::default()
            },
            TestNode {
                name: "section",
                entries: entries("\"First section\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the first paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the second paragraph\"")
            },
            TestNode {
                name: "contents",
                entries: Entries::default()
            },
            TestNode {
                name: "section",
                entries: entries("\"Second section\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the third paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the forth paragraph\"")
            },
        ])
    );
    assert_eq!(
        Resolver::resolve(&*KDL_DOC, Path::parse("article/**/paragraph").unwrap()),
        TestNodes(vec![
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the first paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the second paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the third paragraph\"")
            },
            TestNode {
                name: "paragraph",
                entries: entries("\"This is the forth paragraph\"")
            },
        ])
    );
}
