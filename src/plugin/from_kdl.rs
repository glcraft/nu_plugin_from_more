use super::FromMorePlugin;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use miette::NarratableReportHandler;
use nu_plugin::EvaluatedCall;
use nu_plugin::{EngineInterface, PluginCommand, SimplePluginCommand};
use nu_protocol::{LabeledError, Record, Signature, Span, Type, Value as NuValue};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse KDL document: {}", Self::from_kdl_error(.0))]
    KdlParseError(#[from] kdl::KdlError),

    #[error("Unable to convert from {from} to {to} (value: {value:?})")]
    BadConversion {
        from: String,
        to: String,
        value: Option<String>,
    },
}

impl Error {
    fn from_kdl_error(error: &kdl::KdlError) -> String {
        let report_handler = NarratableReportHandler::new();

        let mut rendered = error
            .diagnostics
            .iter()
            .fold(String::new(), |mut acc, diag| {
                let _ = report_handler.render_report(&mut acc, diag);
                acc
            });
        rendered.pop();
        rendered
    }
}

pub struct FromKdl;

impl SimplePluginCommand for FromKdl {
    type Plugin = FromMorePlugin;

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
        _plugin: &FromMorePlugin,
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
                    .with_label(e.to_string(), call.head)
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

type ResultValue = Result<NuValue, Error>;

impl FromKdl {
    fn parse(input: &str) -> ResultValue {
        let kdoc: KdlDocument = input.parse()?;
        Self::convert_document(&kdoc)
    }
    fn convert_value(kvalue: &KdlValue) -> ResultValue {
        Ok(match kvalue {
            KdlValue::String(s) => NuValue::string(s, Span::unknown()),
            KdlValue::Integer(i) => NuValue::int(
                (*i).try_into().map_err(|_| Error::BadConversion {
                    from: "i128".into(),
                    to: "i64".into(),
                    value: Some(i.to_string()),
                })?,
                Span::unknown(),
            ),
            KdlValue::Float(v) => NuValue::float(*v, Span::unknown()),
            KdlValue::Bool(v) => NuValue::bool(*v, Span::unknown()),
            KdlValue::Null => NuValue::nothing(Span::unknown()),
        })
    }
    fn convert_entries(kentries: &[KdlEntry]) -> Result<(Vec<NuValue>, Record), Error> {
        let args_len = kentries
            .iter()
            .filter(|entry| entry.name().is_some())
            .count();
        let props_len = kentries.len() - args_len;
        let result = kentries.iter().fold(
            (
                Vec::with_capacity(args_len),
                Record::with_capacity(props_len),
            ),
            |(mut args, mut props), entry| {
                let value = Self::convert_value(entry.value()).unwrap();
                match entry.name() {
                    Some(name) => {
                        props.insert(name.value(), value);
                    }
                    None => args.push(value),
                }
                (args, props)
            },
        );
        Ok(result)
    }
    fn convert_document(kdoc: &KdlDocument) -> ResultValue {
        let nu_nodes = kdoc
            .nodes()
            .iter()
            .map(Self::convert_node)
            .collect::<Result<Vec<NuValue>, Error>>()?;
        Ok(NuValue::list(nu_nodes, Span::unknown()))
    }
    fn convert_node(knode: &KdlNode) -> ResultValue {
        let mut rec = Record::new();
        rec.push(
            "name",
            NuValue::string(knode.name().value(), Span::unknown()),
        );
        if knode.entries().len() > 0 {
            let (args, props) = Self::convert_entries(knode.entries())?;
            if args.len() > 0 {
                rec.push("arguments", NuValue::list(args, Span::unknown()))
            }
            if props.len() > 0 {
                rec.push("properties", NuValue::record(props, Span::unknown()))
            }
        }
        if let Some(kdoc) = knode.children() {
            rec.push("children", Self::convert_document(kdoc)?);
        }
        Ok(NuValue::record(rec, Span::unknown()))
    }
}
