use crate::parser::{Entries, Node, NodeKind, Path, Value};
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub fn resolve_document<'a, 'b>(doc: &'a KdlDocument, query: Path<'b>) -> Vec<&'a KdlNode> {
    todo!()
}

mod ops {
    use crate::parser::{Entries, EntryKind, Node, NodeKind, Path, Value};
    use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
    impl<'a, 'b> PartialEq<Entries<'a>> for &'b [KdlEntry] {
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
    fn resolve_impl<'b>(&mut self, query: &'b [Node], node: &'a KdlNode) {
        let kdl_doc = self
            .current_node
            .last()
            .map(|v| v.children())
            .unwrap_or_else(|| Some(self.doc));
        let Some(kdl_doc) = kdl_doc else {
            return;
        };
        let kdl_nodes = kdl_doc.nodes();
        if kdl_nodes.is_empty() {
            return;
        }
        let query_node = query.first().unwrap();
        // match query_node.node {
        //     NodeKind::Named(name) => kdl_nodes
        //         .iter()
        //         .filter(|node| node.name() == name)
        //         .for_each(|node| self.resolve_impl()),
        // }
        todo!()
    }
}
