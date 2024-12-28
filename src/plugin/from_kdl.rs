use super::FromAdvPlugin;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use nu_plugin::EvaluatedCall;
use nu_plugin::{EngineInterface, PluginCommand, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, Type, Value as NuValue};

pub struct FromKdl;

impl SimplePluginCommand for FromKdl {
    type Plugin = FromAdvPlugin;

    fn name(&self) -> &str {
        "from kdl"
    }

    fn description(&self) -> &str {
        "parse kdl structured data"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::String, Type::List(Type::Any.into()))
    }

    fn run(
        &self,
        _plugin: &FromAdvPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &NuValue,
    ) -> Result<NuValue, LabeledError> {
        // if let NuValue::String { val, .. } = input {
        //     println!("INPUT: {}", val);
        //     return Ok(NuValue::nothing(Span::unknown()));
        // }
        match input {
            NuValue::String { val, .. } => Self::parse(&val).map_err(|e| {
                LabeledError::new("Unable to convert kdl input to nu value")
                    .with_label(e, call.head)
            }),
            _ => Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            ),
        }
    }
}

impl FromKdl {
    fn parse(input: &str) -> Result<NuValue, String> {
        let kdoc: KdlDocument = input
            .parse()
            .map_err(|e| format!("Unable to parse kdl from input string: {e:#?}"))?;
        Self::convert_document(&kdoc)
    }
    fn convert_value(kvalue: &KdlValue) -> Result<NuValue, String> {
        Ok(match kvalue {
            KdlValue::String(s) => NuValue::string(s, Span::unknown()),
            KdlValue::Integer(i) => NuValue::int(
                (*i).try_into()
                    .map_err(|_| "out of bound integer".to_string())?,
                Span::unknown(),
            ),
            KdlValue::Float(v) => NuValue::float(*v, Span::unknown()),
            KdlValue::Bool(v) => NuValue::bool(*v, Span::unknown()),
            KdlValue::Null => NuValue::nothing(Span::unknown()),
        })
    }
    fn convert_entries(kentries: &[KdlEntry]) -> Result<NuValue, String> {
        let nu_entries = kentries
            .iter()
            .map(|kentry| -> Result<NuValue, String> {
                let mut rec = Record::new();
                rec.push(
                    "name",
                    kentry
                        .name()
                        .map(|ident| NuValue::string(ident.value().to_string(), Span::unknown()))
                        .unwrap_or(NuValue::nothing(Span::unknown())),
                );
                rec.push("value", Self::convert_value(kentry.value())?);
                Ok(NuValue::record(rec, Span::unknown()))
            })
            .collect::<Result<Vec<NuValue>, String>>()?;
        Ok(NuValue::list(nu_entries, Span::unknown()))
    }
    fn convert_document(kdoc: &KdlDocument) -> Result<NuValue, String> {
        let nu_nodes = kdoc
            .nodes()
            .iter()
            .map(Self::convert_node)
            .collect::<Result<Vec<NuValue>, String>>()?;
        Ok(NuValue::list(nu_nodes, Span::unknown()))
    }
    fn convert_node(knode: &KdlNode) -> Result<NuValue, String> {
        let mut rec = Record::new();
        rec.push(
            "name",
            NuValue::string(knode.name().value(), Span::unknown()),
        );
        if knode.entries().len() > 0 {
            rec.push("entries", Self::convert_entries(knode.entries())?);
        }
        if let Some(kdoc) = knode.children() {
            rec.push("children", Self::convert_document(kdoc)?);
        }
        Ok(NuValue::record(rec, Span::unknown()))
    }
}
