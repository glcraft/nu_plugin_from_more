use std::ops::Index;

use kdl::KdlNode;
use std::iter;
type IterNodes<'a> =
    std::iter::Map<std::slice::Iter<'a, KdlNode>, fn(&'a KdlNode) -> AnywhereIter<'a>>;
#[derive(Clone, Debug)]
enum AnywhereIter<'a> {
    Node(&'a KdlNode),
    Children(IterNodes<'a>, Box<AnywhereIter<'a>>),
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
                        let i: IterNodes = kdl_doc.nodes().iter().map(AnywhereIter::Node);
                        let mut it = kdl_doc.nodes().iter();
                        let Some(child) = it.next().map(AnywhereIter::Node) else {
                            return AnywhereIter::End;
                        };
                        AnywhereIter::Children(it, Box::new(child))
                    })
                    .unwrap_or(AnywhereIter::End),
                Some(kdl_node),
            ),
            AnywhereIter::Children(ref mut it_nodes, ref mut actual) => {
                if let Some(kdl_node) = actual.next() {
                    return Some(kdl_node);
                }
                todo!()
            }
            AnywhereIter::End => (AnywhereIter::End, None),
        };
        *self = new_self;
        output
    }
}
