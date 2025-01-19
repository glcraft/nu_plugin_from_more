mod lexer;
mod parser;
use super::FromMorePlugin;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::NarratableReportHandler;
use nu_plugin::EvaluatedCall;
use nu_plugin::{EngineInterface, PluginCommand, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, SyntaxShape, Type, Value as NuValue};
use query_kdl::Path;
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
        let query = parser::Path::parse(&query_str).map_err(|e| {
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
