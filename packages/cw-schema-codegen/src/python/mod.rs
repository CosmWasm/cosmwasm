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
            format!("{}[]", expand_node_name(schema, items)).into()
        }
        cw_schema::NodeType::Float => "float".into(),
        cw_schema::NodeType::Double => "float".into(),
        cw_schema::NodeType::Boolean => "bool".into(),
        cw_schema::NodeType::String => "str".into(),
        cw_schema::NodeType::Integer { .. } => "int".into(),
        cw_schema::NodeType::Binary => "bytes".into(),
        cw_schema::NodeType::Optional { inner } => {
            let inner = &schema.definitions[inner];
            format!("typing.Optional[{}]", expand_node_name(schema, inner)).into()
        }
        cw_schema::NodeType::Struct(..) => node.name.as_ref().into(),
        cw_schema::NodeType::Tuple { ref items } => {
            let items = items
                .iter()
                .map(|item| expand_node_name(schema, &schema.definitions[*item]))
                .collect::<Vec<_>>()
                .join(", ");

            format!("[{}]", items).into()
        }
        cw_schema::NodeType::Enum { .. } => node.name.as_ref().into(),

        cw_schema::NodeType::Decimal { .. } => "decimal.Decimal".into(),
        cw_schema::NodeType::Address => "str".into(),
        cw_schema::NodeType::Checksum => todo!(),
        cw_schema::NodeType::HexBinary => todo!(),
        cw_schema::NodeType::Timestamp => todo!(),
        cw_schema::NodeType::Unit => "None".into(),
        _ => todo!(),
    }
}

fn prepare_docs(desc: Option<&str>) -> Cow<'_, [Cow<'_, str>]> {
    desc.map(|desc| desc.lines().map(Into::into).collect())
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
                name: node.name.clone(),
                docs: prepare_docs(node.description.as_deref()),
                ty: match sty {
                    cw_schema::StructType::Unit => TypeTemplate::Unit,
                    cw_schema::StructType::Named { ref properties } => TypeTemplate::Named {
                        fields: properties
                            .iter()
                            .map(|(name, prop)| FieldTemplate {
                                name: Cow::Borrowed(name),
                                docs: prepare_docs(prop.description.as_deref()),
                                ty: expand_node_name(schema, &schema.definitions[prop.value]),
                            })
                            .collect(),
                    },
                    cw_schema::StructType::Tuple { ref items } => TypeTemplate::Tuple(
                        items
                            .iter()
                            .map(|item| expand_node_name(schema, &schema.definitions[*item]))
                            .collect(),
                    ),
                    _ => todo!(),
                },
            };

            writeln!(output, "{structt}")?;
        }
        cw_schema::NodeType::Enum { ref cases, .. } => {
            let enumm = EnumTemplate {
                name: node.name.clone(),
                docs: prepare_docs(node.description.as_deref()),
                variants: cases
                    .iter()
                    .map(|(name, case)| EnumVariantTemplate {
                        name: name.clone(),
                        docs: prepare_docs(case.description.as_deref()),
                        ty: match case.value {
                            cw_schema::EnumValue::Unit => TypeTemplate::Unit,
                            cw_schema::EnumValue::Tuple { ref items } => {
                                let items = items
                                    .iter()
                                    .map(|item| {
                                        expand_node_name(schema, &schema.definitions[*item])
                                    })
                                    .collect();

                                TypeTemplate::Tuple(items)
                            }
                            cw_schema::EnumValue::Named { ref properties, .. } => {
                                TypeTemplate::Named {
                                    fields: properties
                                        .iter()
                                        .map(|(name, prop)| FieldTemplate {
                                            name: Cow::Borrowed(name),
                                            docs: prepare_docs(prop.description.as_deref()),
                                            ty: expand_node_name(
                                                schema,
                                                &schema.definitions[prop.value],
                                            ),
                                        })
                                        .collect(),
                                }
                            }
                            _ => todo!(),
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
