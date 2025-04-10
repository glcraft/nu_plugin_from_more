use std::ops::Index;

use kdl::KdlNode;
use std::iter;

type IterChildren<'a> =
    iter::Map<std::slice::Iter<'a, KdlNode>, fn(&'a KdlNode) -> AnywhereIter<'a>>;
#[derive(Clone, Debug)]
enum AnywhereIter<'a> {
    Node(&'a KdlNode),
    Children(Vec<AnywhereIter<'a>>, &AnywhereIter<'a>, usize),
    End,
}

impl<'a> Iterator for AnywhereIter<'a> {
    type Item = &'a KdlNode;

    fn next(&mut self) -> Option<Self::Item> {
        let (new_self, output) = match *self {
            AnywhereIter::Node(kdl_node) => (
                kdl_node
                    .children()
                    .map(|kdl_doc| {
                        let i: IterChildren = kdl_doc.nodes().iter().map(AnywhereIter::Node);
                        AnywhereIter::Children(
                            kdl_doc
                                .nodes()
                                .iter()
                                .map(|v| AnywhereIter::Node(v))
                                .collect(),
                            0,
                        )
                    })
                    .unwrap_or(AnywhereIter::End),
                Some(kdl_node),
            ),
            AnywhereIter::Children(kdl_nodes, index) => (
                if index < kdl_nodes.len() {
                    AnywhereIter::Children(kdl_nodes, index + 1)
                } else {
                    AnywhereIter::End
                },
                todo!(),
            ),
            AnywhereIter::End => (AnywhereIter::End, None),
        };
        *self = new_self;
        output
    }
}
