mod ops;
#[cfg(test)]
mod tests;
use crate::parser::{Entries, Node, NodeKind, Path, Range, Value};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn resolve_document<'a, 'b>(doc: &'a KdlDocument, query: Path<'b>) -> Vec<&'a KdlNode> {
    todo!()
}

struct Resolver<'a> {
    doc: &'a KdlDocument,
    current_node: Vec<&'a KdlDocument>,
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
        // let kdl_nodes =
        self.resolve_impl(query.nodes(), self.doc);
        &self.found_nodes
    }
    fn resolve_impl<'b>(&mut self, query: &'b [Node], kdl_doc: &'a KdlDocument) {
        let query_node = unsafe { query.last().unwrap_unchecked() };
        let compare_entries = |entries: &[KdlEntry]| {
            query_node
                .entries
                .as_ref()
                .map(|query_entries| entries == query_entries)
                .or(Some(true))
                == Some(true)
        };
        // let Some(kdl_doc) = kdl_node.children() else {
        //     return;
        // };
        let kdl_nodes = kdl_doc.nodes();
        match &query_node.node {
            NodeKind::Named(name) => {
                let it = kdl_nodes.iter().filter(|kdl_node| {
                    kdl_node.name().repr().unwrap() == name && compare_entries(kdl_node.entries())
                });
                self.check(&query[1..], Some(kdl_doc), it);
            }
            NodeKind::Any => {
                let it = kdl_nodes
                    .iter()
                    .filter(|kdl_node| compare_entries(kdl_node.entries()));
                self.check(&query[1..], Some(kdl_doc), it);
            }
            NodeKind::Anywhere => unimplemented!(),
            NodeKind::Parent => {
                let kdl_node_parent = self.current_node.pop();
                let Some(kdl_node_parent) = kdl_node_parent else {
                    return;
                };
                // if compare_entries(kdl_node_parent.entries()) {
                // self.check(&query[1..], None, std::iter::once(kdl_node_parent));
                todo!("special case");
                // }
                self.current_node.push(kdl_node_parent);
            }
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
                    Some(kdl_doc),
                    it.filter(|node| compare_entries(node.entries())),
                );
            }
        }
    }
    #[inline]
    fn check<'b>(
        &mut self,
        query: &'b [Node],
        kdl_node_parent: Option<&'a KdlDocument>,
        kdl_nodes: impl Iterator<Item = &'a KdlNode>,
    ) {
        if query.is_empty() {
            kdl_nodes.for_each(|kdl_node| self.found_nodes.push(kdl_node));
        } else {
            let has_parent = kdl_node_parent.is_some();
            if let Some(parent) = kdl_node_parent {
                self.current_node.push(parent);
            }
            for kdl_node in kdl_nodes {
                if let Some(kdl_doc) = kdl_node.children() {
                    self.resolve_impl(query, kdl_doc);
                }
            }
            if has_parent {
                let _ = self.current_node.pop();
            }
        }
    }
}
