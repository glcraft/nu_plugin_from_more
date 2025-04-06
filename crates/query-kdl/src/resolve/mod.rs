mod ops;
#[cfg(test)]
mod tests;
use crate::parser::{Entries, Node, NodeKind, Path, Range, Value};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn resolve_document<'a, 'b>(doc: &'a KdlDocument, query: Path<'b>) -> Vec<&'a KdlNode> {
    todo!()
}

enum NodeOrDoc<'a> {
    Node(&'a KdlNode),
    Doc(&'a KdlDocument),
}

impl<'a> From<&'a KdlNode> for NodeOrDoc<'a> {
    fn from(value: &'a KdlNode) -> Self {
        Self::Node(value)
    }
}
impl<'a> From<&'a KdlDocument> for NodeOrDoc<'a> {
    fn from(value: &'a KdlDocument) -> Self {
        Self::Doc(value)
    }
}

impl<'a> NodeOrDoc<'a> {
    fn document(&self) -> Option<&'a KdlDocument> {
        match self {
            NodeOrDoc::Node(kdl_node) => kdl_node.children(),
            NodeOrDoc::Doc(kdl_document) => Some(&kdl_document),
        }
    }
}

struct Resolver<'a> {
    doc: &'a KdlDocument,
    current_node: Vec<NodeOrDoc<'a>>,
    found_nodes: Vec<&'a KdlNode>,
}

impl<'a> From<&'a KdlDocument> for Resolver<'a> {
    fn from(doc: &'a KdlDocument) -> Self {
        Self {
            doc,
            current_node: Vec::new(),
            found_nodes: Vec::new(),
        }
    }
}

impl<'a> Resolver<'a> {
    fn resolve<'b>(&mut self, query: Path<'b>) -> &Vec<&'a KdlNode> {
        self.current_node.clear();
        self.found_nodes.clear();
        self.resolve_impl(query.nodes(), self.doc.into());
        &self.found_nodes
    }
    fn resolve_impl<'b>(&mut self, query: &'b [Node], kdl_orig: NodeOrDoc<'a>) {
        let query_node = unsafe { query.first().unwrap_unchecked() };
        let compare_entries = |entries: &[KdlEntry]| {
            query_node
                .entries
                .as_ref()
                .map(|query_entries| entries == query_entries)
                .unwrap_or(true)
        };
        if matches!(&query_node.node, NodeKind::Parent) {
            let kdl_node_parent = self.current_node.pop();
            let Some(kdl_node_parent) = kdl_node_parent else {
                return;
            };
            let NodeOrDoc::Node(kdl_node_parent) = kdl_node_parent else {
                self.current_node.push(kdl_node_parent);
                return;
            };
            self.check(&query[1..], None, std::iter::once(kdl_node_parent));
            self.current_node.push(NodeOrDoc::Node(kdl_node_parent));
            return;
        }
        let Some(kdl_doc) = kdl_orig.document() else {
            return;
        };
        let kdl_nodes = kdl_doc.nodes();
        match &query_node.node {
            NodeKind::Named(query_name) => {
                let it = kdl_nodes.iter().filter(|kdl_node| {
                    kdl_node
                        .name()
                        .repr()
                        .map(|node_name| node_name == query_name)
                        .unwrap_or(false)
                        && compare_entries(kdl_node.entries())
                });

                self.check(&query[1..], Some(kdl_orig), it);
            }
            NodeKind::Any => {
                let it = kdl_nodes
                    .iter()
                    .filter(|kdl_node| compare_entries(kdl_node.entries()));
                self.check(&query[1..], Some(kdl_orig), it);
            }
            NodeKind::Anywhere => unimplemented!(),
            NodeKind::Parent => unreachable!("already defined on top"),
            NodeKind::Ranged(range) => {
                let it = kdl_nodes.iter();
                let it = match range {
                    Range::One(i) => it.skip(*i as _).take(1),
                    Range::From(from) => it.skip(*from as _).take(usize::MAX),
                    Range::To(to) => it.skip(0).take(*to as _),
                    Range::Both(from, to) => it.skip(*from as _).take((to - from) as _),
                    Range::All => it.skip(0).take(usize::MAX),
                };

                self.check(
                    &query[1..],
                    Some(kdl_orig),
                    it.filter(|node| compare_entries(node.entries())),
                );
            }
        }
    }
    #[inline]
    fn check<'b>(
        &mut self,
        query: &'b [Node],
        kdl_node_parent: Option<NodeOrDoc<'a>>,
        kdl_nodes: impl Iterator<Item = &'a KdlNode> + Clone,
    ) {
        if query.is_empty() {
            kdl_nodes.for_each(|kdl_node| self.found_nodes.push(kdl_node));
            return;
        }
        let has_parent = kdl_node_parent.is_some();
        if let Some(parent) = kdl_node_parent {
            self.current_node.push(parent);
        }
        for kdl_node in kdl_nodes {
            self.resolve_impl(query, kdl_node.into());
        }
        if has_parent {
            let _ = self.current_node.pop();
        }
    }
}
