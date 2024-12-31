use super::FromMorePlugin;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::NarratableReportHandler;
use nu_plugin::EvaluatedCall;
use nu_plugin::{EngineInterface, PluginCommand, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, SyntaxShape, Type, Value as NuValue};
use std::collections::HashMap;

pub struct QueryKdl;

impl SimplePluginCommand for QueryKdl {
    type Plugin = FromMorePlugin;

    fn name(&self) -> &str {
        "query kdl"
    }

    fn description(&self) -> &str {
        "select kdl data"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::String, Type::String)
            .required("query", SyntaxShape::String, "xpath like query")
    }

    fn run(
        &self,
        _plugin: &FromMorePlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &NuValue,
    ) -> Result<NuValue, LabeledError> {
        let query = call.positional.get(0).ok_or(
            LabeledError::new("Missing argument").with_label("query is missing", call.head),
        )?;

        let NuValue::String { val: input, .. } = input else {
            return Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            );
        };
        let query_span = query.span();
        let NuValue::String { val: query_str, .. } = query else {
            return Err(LabeledError::new("Expected String for query").with_label(
                format!("Expected a string, got {}", query.get_type()),
                query_span,
            ));
        };
        let query = Path::parse(&query_str).map_err(|e| {
            LabeledError::new("Failed to parse query").with_label(e.to_string(), query_span)
        })?;

        let kdoc = match KdlDocument::parse(input) {
            Ok(v) => v,
            Err(e) => {
                return Err(LabeledError::new("Failed to parse KDL format")
                    .with_label(e.to_string(), call.head))
            }
        };
        todo!()
    }
}

impl QueryKdl {}

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

struct Lexer<'a> {
    input: &'a str,
}

impl<'a, T: AsRef<str>> From<&'a T> for Lexer<'a> {
    fn from(value: &'a T) -> Self {
        Self {
            input: value.as_ref(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespaces
        let byte_offset = self
            .input
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum();
        self.input = &self.input[byte_offset..];
        let c_token = self.input.chars().next();
        match c_token {
            None => None,
            Some('"' | '\'') => self.get_text(),
            Some(c) if c.is_alphanumeric() => self.get_alphanumeric(),
            Some(_) => self.get_token(),
        }
    }
}
impl<'a> Lexer<'a> {
    fn get_text(&mut self) -> Option<<Self as Iterator>::Item> {
        let iter_chars = self.input.chars();
        todo!()
    }
    fn get_alphanumeric(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!()
    }
    fn get_token(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!()
    }
}
