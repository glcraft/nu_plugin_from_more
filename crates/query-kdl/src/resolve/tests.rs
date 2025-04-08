use kdl::{KdlDocument, KdlNode};
use std::sync::LazyLock;

use crate::{
    lexer::Lexer,
    parser::{Entries, Path},
    resolve::Resolver,
};

static KDL_DOC: LazyLock<KdlDocument> = LazyLock::new(|| {
    r#"
        node1
        node2 1 2 3
        node2
        node3 a b c
        node3 0 2 0
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
        node4 {
            contents {
                section "First section" {
                    paragraph "This is the first paragraph"
                    paragraph "This is the second paragraph"
                }
            }
            contents {
                section "First section" {
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
