use crate::parser::{Entries, Node, NodeKind, Path, Range, Value};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn resolve_document<'a, 'b>(doc: &'a KdlDocument, query: Path<'b>) -> Vec<&'a KdlNode> {
    todo!()
}

mod ops {
    use crate::parser::{Entries, EntryKind, Node, NodeKind, Path, Value};
    use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
    impl<'a> PartialEq<Entries<'a>> for [KdlEntry] {
        fn eq(&self, other: &Entries<'a>) -> bool {
            let entries = other.entries();
            for entry in entries {
                match entry {
                    EntryKind::Argument { position, value } => {
                        let Ok(pos): Result<usize, _> = (*position).try_into() else {
                            return false;
                        };
                        let Some(arg) = self.get(pos) else {
                            return false;
                        };
                        if arg.value() != value {
                            return false;
                        }
                    }
                    EntryKind::Property { name, value } => {
                        let Some(prop) = self
                            .iter()
                            .filter_map(|v| v.name().map(|name| (v, name)))
                            .find(|v| v.1.value() == name)
                            .map(|v| v.0)
                        else {
                            return false;
                        };
                        if prop.value() != value {
                            return false;
                        }
                    }
                }
            }
            true
        }
    }

    impl<'a> PartialEq<Value<'a>> for KdlValue {
        fn eq(&self, other: &Value<'a>) -> bool {
            match (self, other) {
                (KdlValue::String(v1), Value::String(v2)) => v1 == v2,
                (KdlValue::Float(v1), Value::FloatingPoing(v2)) => v1 == v2,
                (KdlValue::Integer(v1), Value::Integer(v2)) => v1 == v2,
                (KdlValue::Bool(v1), Value::Boolean(v2)) => v1 == v2,
                (KdlValue::Null, Value::Null) => true,
                _ => false,
            }
        }
    }
}

struct Resolver<'a> {
    doc: &'a KdlDocument,
    current_node: Vec<&'a KdlNode>,
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
    fn resolve<'b>(&mut self, query: Path<'b>) -> Vec<&'a KdlNode> {
        todo!()
    }
    fn resolve_impl<'b>(&mut self, query: &'b [Node], kdl_node: &'a KdlNode) {
        let query_node = unsafe { query.last().unwrap_unchecked() };
        let compare_entries = |entries: &[KdlEntry]| {
            query_node
                .entries
                .as_ref()
                .map(|query_entries| entries == query_entries)
                == Some(true)
        };
        match &query_node.node {
            NodeKind::Named(name) => {
                let it = kdl_nodes.iter().filter(|kdl_node| {
                    kdl_node.name().repr().unwrap() == name && compare_entries(kdl_node.entries())
                });
                self.check(&query[1..], it);
            }
            NodeKind::Any => kdl_nodes
                .iter()
                .for_each(|kdl_node| compare_entries(kdl_node.entries())),
            NodeKind::Anywhere => unimplemented!(),
            NodeKind::Parent => {
                let kdl_node_parent = self.current_node.pop();
                let Some(kdl_node_parent) = kdl_node_parent else {
                    return;
                };
                if compare_entries(kdl_node_parent.entries()) {
                    // self.check(&query[1..], , std::iter::once(kdl_node_parent));
                    todo!("special case")
                }
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
                    node,
                    it.filter(|node| compare_entries(node.entries())),
                );
            }
        }
        todo!()
    }
    #[inline]
    fn check<'b>(
        &mut self,
        query: &'b [Node],
        kdl_node_parent: &'a KdlNode,
        kdl_nodes: impl Iterator<Item = &'a KdlNode>,
    ) {
        if query.is_empty() {
            kdl_nodes.for_each(|kdl_node| self.found_nodes.push(kdl_node));
        } else {
            self.current_node.push(kdl_node_parent);
            for kdl_node in kdl_nodes {
                self.resolve_impl(query, kdl_node);
            }
            let _ = self.current_node.pop();
        }
    }
}
