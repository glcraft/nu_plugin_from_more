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
        node_prop hello=world foo=bar
        node4 {
            contents {
                section "First section" {
                    paragraph "This is the first paragraph"
                    paragraph "This is the second paragraph"
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
fn document() {
    let query = Path::parse("node2").unwrap();
    let mut resolver = Resolver::from(&*KDL_DOC);
    resolver.resolve(query);
    assert_eq!(
        resolver.found_nodes,
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
