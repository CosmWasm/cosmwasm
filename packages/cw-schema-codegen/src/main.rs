#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use clap::{Parser, ValueEnum};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum Language {
    #[default]
    Rust,
    Go,
    Typescript,
}

#[derive(Parser)]
#[clap(about, author, version)]
/// Official CosmWasm codegen tool
struct Opts {
    #[clap(default_value_t, long, short, value_enum)]
    /// Programming language to generate code for
    language: Language,

    #[clap(long, short)]
    /// Path to the schema file
    file: PathBuf,

    #[clap(long, short)]
    /// Path to the output file
    output: Option<PathBuf>,
}

impl Opts {
    fn output(&self) -> anyhow::Result<impl Write> {
        let io_out = if let Some(ref path) = self.output {
            either::Left(File::create(path)?)
        } else {
            either::Right(io::stdout().lock())
        };

        Ok(io_out)
    }
}

fn expand_node_name<'a>(
    schema: &'a cw_schema::SchemaV1,
    node: &'a cw_schema::Node,
) -> Cow<'a, str> {
    match node.value {
        cw_schema::NodeType::Array { items } => {
            let items = &schema.definitions[items];
            format!("Vec<{}>", expand_node_name(schema, items)).into()
        }
        cw_schema::NodeType::Float => "f32".into(),
        cw_schema::NodeType::Double => "f64".into(),
        cw_schema::NodeType::Boolean => "bool".into(),
        cw_schema::NodeType::String => "String".into(),
        cw_schema::NodeType::Integer { signed, precision } => {
            let ty = if signed { "i" } else { "u" };
            format!("{ty}{precision}").into()
        }
        cw_schema::NodeType::Binary => "Vec<u8>".into(),
        cw_schema::NodeType::Optional { inner } => {
            let inner = &schema.definitions[inner];
            format!("Option<{}>", expand_node_name(schema, inner)).into()
        }
        cw_schema::NodeType::Struct(..) => node.name.as_ref().into(),
        cw_schema::NodeType::Tuple { ref items } => {
            let items = items
                .iter()
                .map(|item| expand_node_name(schema, &schema.definitions[*item]))
                .collect::<Vec<_>>()
                .join(", ");

            format!("({})", items).into()
        }
        cw_schema::NodeType::Enum { .. } => node.name.as_ref().into(),
        _ => todo!(),
    }
}

fn prepare_docs(desc: Option<&str>) -> Cow<'_, [Cow<'_, str>]> {
    desc.map(|desc| {
        desc.lines()
            .map(|line| line.replace('"', "\\\"").into())
            .collect()
    })
    .unwrap_or(Cow::Borrowed(&[]))
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()?;

    let opts: Opts = Opts::parse();
    info!(
        "Generating code for {:?} from {:?}",
        opts.language, opts.file
    );

    ensure!(opts.file.exists(), "Schema file does not exist");
    ensure!(
        opts.language == Language::Rust,
        "Only Rust code generation is supported at the moment"
    );

    let schema = fs::read_to_string(&opts.file)?;
    let schema: cw_schema::Schema = serde_json::from_str(&schema)?;
    let cw_schema::Schema::V1(schema) = schema else {
        bail!("Unsupported schema version");
    };

    let mut output = opts.output()?;

    schema.definitions.iter().for_each(|node| {
        debug!("Processing node: {node:?}");

        match node.value {
            cw_schema::NodeType::Struct(ref sty) => {
                let structt = cw_schema_codegen::rust::StructTemplate {
                    name: &node.name,
                    docs: prepare_docs(node.description.as_deref()),
                    ty: match sty {
                        cw_schema::StructType::Unit => cw_schema_codegen::rust::TypeTemplate::Unit,
                        cw_schema::StructType::Named { ref properties } => {
                            cw_schema_codegen::rust::TypeTemplate::Named {
                                fields: properties
                                    .iter()
                                    .map(|(name, prop)| {
                                        let ty = expand_node_name(
                                            &schema,
                                            &schema.definitions[prop.value],
                                        );
                                        cw_schema_codegen::rust::FieldTemplate {
                                            name: Cow::Borrowed(name),
                                            docs: prepare_docs(prop.description.as_deref()),
                                            ty,
                                        }
                                    })
                                    .collect(),
                            }
                        }
                        _ => unreachable!(),
                    },
                };

                writeln!(output, "{structt}").unwrap();
            }
            cw_schema::NodeType::Enum { ref cases, .. } => {
                let enumm = cw_schema_codegen::rust::EnumTemplate {
                    name: &node.name,
                    docs: prepare_docs(node.description.as_deref()),
                    variants: cases
                        .iter()
                        .map(
                            |(name, case)| cw_schema_codegen::rust::EnumVariantTemplate {
                                name,
                                docs: prepare_docs(case.description.as_deref()),
                                ty: match case.value {
                                    cw_schema::EnumValue::Unit => {
                                        cw_schema_codegen::rust::TypeTemplate::Unit
                                    }
                                    cw_schema::EnumValue::Tuple { ref items } => {
                                        let items = items
                                            .iter()
                                            .map(|item| {
                                                let node = &schema.definitions[*item];
                                                expand_node_name(&schema, node)
                                            })
                                            .collect();

                                        cw_schema_codegen::rust::TypeTemplate::Tuple(items)
                                    }
                                    cw_schema::EnumValue::Named { ref properties, .. } => {
                                        cw_schema_codegen::rust::TypeTemplate::Named {
                                            fields: properties
                                                .iter()
                                                .map(|(name, prop)| {
                                                    let ty = expand_node_name(
                                                        &schema,
                                                        &schema.definitions[prop.value],
                                                    );
                                                    cw_schema_codegen::rust::FieldTemplate {
                                                        name: Cow::Borrowed(name),
                                                        docs: prepare_docs(
                                                            prop.description.as_deref(),
                                                        ),
                                                        ty,
                                                    }
                                                })
                                                .collect(),
                                        }
                                    }
                                    _ => unreachable!(),
                                },
                            },
                        )
                        .collect(),
                };

                writeln!(output, "{enumm}").unwrap();
            }
            _ => (),
        }
    });

    Ok(())
}
