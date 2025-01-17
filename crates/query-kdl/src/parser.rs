use std::collections::HashMap;

enum NodeIdentifier<'a> {
    /// Node with a name
    Named(&'a str),
    /// Any nodes in the current scope
    Any,
    /// Root node
    Root,
    /// Nodes starting anywhere in the doc
    Anywhere,
    /// Parent node
    Parent,
}

struct Node<'a> {
    ident: NodeIdentifier<'a>,
    arguments: Vec<&'a str>,
    properties: HashMap<&'a str, &'a str>,
}

struct Path<'a> {
    nodes: Vec<Node<'a>>,
}

#[derive(thiserror::Error, Debug)]
enum ParseQueryError {}

impl<'a> Path<'a> {
    fn parse(input: &'a str) -> Result<Self, ParseQueryError> {
        todo!()
    }
}
