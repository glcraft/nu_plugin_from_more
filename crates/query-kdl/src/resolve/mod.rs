mod ops;
#[cfg(test)]
mod tests;
use kdl::{KdlDocument, KdlEntry, KdlNode};

use crate::parser::{Node as QueryNode, NodeKind, Path};

struct Resolver<'k> {
    current_nodes: Vec<&'k KdlNode>,
    found_nodes: Vec<&'k KdlNode>,
}

impl<'k> Resolver<'k> {
    fn resolve<'q>(kdl_doc: &'k KdlDocument, query: Path<'q>) -> Vec<&'k KdlNode> {
        let mut r = Resolver {
            current_nodes: Vec::with_capacity(query.nodes().len()),
            found_nodes: Vec::new(),
        };
        r.resolve_query_node(query.nodes(), kdl_doc.nodes().iter());
        r.found_nodes
    }
    fn resolve_query_node<'q>(
        &mut self,
        query: &'q [QueryNode],
        it_nodes: impl Iterator<Item = &'k KdlNode>,
    ) {
        let query_node = unsafe { query.first().unwrap_unchecked() };
        let query_next = &query[1..];
        let compare_entries = |entries: &[KdlEntry]| {
            query_node
                .entries
                .as_ref()
                .map(|query_entries| entries == query_entries)
                .unwrap_or(true)
        };

        match &query_node.node {
            NodeKind::Named(query_name) => {
                let it = it_nodes.filter(|kdl_node| {
                    kdl_node
                        .name()
                        .repr()
                        .map(|node_name| node_name == query_name)
                        .unwrap_or(false)
                        && compare_entries(kdl_node.entries())
                });
                self.dispatch(query_next, it);
            }
            NodeKind::Any => self.dispatch(query_next, it_nodes),
            NodeKind::Anywhere => todo!(),
            NodeKind::Parent => {
                let Some(parent) = self.current_nodes.pop() else {
                    return;
                };
                self.dispatch(query_next, std::iter::once(parent));
                self.current_nodes.push(parent);
            }
            NodeKind::Ranged(range) => match range {
                crate::parser::Range::One(index) => {
                    self.dispatch(query_next, it_nodes.skip(*index as _).take(1))
                }
                crate::parser::Range::From(from) => {
                    self.dispatch(query_next, it_nodes.skip(*from as _))
                }
                crate::parser::Range::To(to) => self.dispatch(query_next, it_nodes.take(*to as _)),
                crate::parser::Range::Both(from, to) => self.dispatch(
                    query_next,
                    it_nodes.skip(*from as _).take((*to - *from) as _),
                ),
                crate::parser::Range::All => self.dispatch(query_next, it_nodes),
            },
        }
    }
    fn dispatch<'q>(
        &mut self,
        query: &'q [QueryNode],
        it_nodes: impl Iterator<Item = &'k KdlNode>,
    ) {
        if query.is_empty() {
            it_nodes.for_each(|kdl_node| self.found_nodes.push(&kdl_node));
            return;
        }

        for kdl_node in it_nodes {
            let Some(kdl_doc) = kdl_node.children() else {
                self.resolve_query_node(query, std::iter::empty());
                continue;
            };
            self.current_nodes.push(kdl_node);
            self.resolve_query_node(query, kdl_doc.nodes().iter());
            let _ = self.current_nodes.pop();
        }
    }
}
