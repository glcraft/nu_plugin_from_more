use kdl::KdlNode;

type IterNodes<'a> = std::slice::Iter<'a, KdlNode>;

#[derive(Clone, Debug)]
pub enum AnywhereIter<'a> {
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
                        let mut it = kdl_doc.nodes().iter();
                        let Some(child) = it.next().map(AnywhereIter::Node) else {
                            return AnywhereIter::End;
                        };
                        AnywhereIter::Children(it, Box::new(child))
                    })
                    .unwrap_or(AnywhereIter::End),
                Some(kdl_node),
            ),
            AnywhereIter::Children(ref mut it_nodes, ref mut actual) => 'mat: {
                if let Some(kdl_node) = actual.next() {
                    return Some(kdl_node);
                }
                let Some(new_node) = it_nodes.next() else {
                    break 'mat (AnywhereIter::End, None);
                };
                *actual = Box::new(AnywhereIter::Node(new_node));
                return actual.next();
            }
            AnywhereIter::End => (AnywhereIter::End, None),
        };
        *self = new_self;
        output
    }
}

pub trait AnywhereIterator<'a>
where
    Self: Iterator<Item = &'a KdlNode> + Sized,
{
    fn anywhere_nodes(self) -> impl Iterator<Item = &'a KdlNode> {
        self.map(AnywhereIter::Node).flatten()
    }
}
impl<'a, I> AnywhereIterator<'a> for I where I: Iterator<Item = &'a KdlNode> {}
