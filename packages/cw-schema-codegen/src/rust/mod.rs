use self::template::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, StructTemplate, TypeTemplate,
};
use std::{borrow::Cow, io};

pub mod template;

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

pub fn process_node<O>(
    output: &mut O,
    schema: &cw_schema::SchemaV1,
    node: &cw_schema::Node,
) -> io::Result<()>
where
    O: io::Write,
{
    match node.value {
        cw_schema::NodeType::Struct(ref sty) => {
            let structt = StructTemplate {
                name: &node.name,
                docs: prepare_docs(node.description.as_deref()),
                ty: match sty {
                    cw_schema::StructType::Unit => TypeTemplate::Unit,
                    cw_schema::StructType::Named { ref properties } => TypeTemplate::Named {
                        fields: properties
                            .iter()
                            .map(|(name, prop)| {
                                let ty = expand_node_name(schema, &schema.definitions[prop.value]);
                                FieldTemplate {
                                    name: Cow::Borrowed(name),
                                    docs: prepare_docs(prop.description.as_deref()),
                                    ty,
                                }
                            })
                            .collect(),
                    },
                    _ => unreachable!(),
                },
            };

            writeln!(output, "{structt}")?;
        }
        cw_schema::NodeType::Enum { ref cases, .. } => {
            let enumm = EnumTemplate {
                name: &node.name,
                docs: prepare_docs(node.description.as_deref()),
                variants: cases
                    .iter()
                    .map(|(name, case)| EnumVariantTemplate {
                        name,
                        docs: prepare_docs(case.description.as_deref()),
                        ty: match case.value {
                            cw_schema::EnumValue::Unit => TypeTemplate::Unit,
                            cw_schema::EnumValue::Tuple { ref items } => {
                                let items = items
                                    .iter()
                                    .map(|item| {
                                        let node = &schema.definitions[*item];
                                        expand_node_name(schema, node)
                                    })
                                    .collect();

                                TypeTemplate::Tuple(items)
                            }
                            cw_schema::EnumValue::Named { ref properties, .. } => {
                                TypeTemplate::Named {
                                    fields: properties
                                        .iter()
                                        .map(|(name, prop)| {
                                            let ty = expand_node_name(
                                                schema,
                                                &schema.definitions[prop.value],
                                            );
                                            FieldTemplate {
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
                    })
                    .collect(),
            };

            writeln!(output, "{enumm}")?;
        }
        _ => (),
    }

    Ok(())
}
