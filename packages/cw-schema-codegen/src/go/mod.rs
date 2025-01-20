use self::template::{
    EnumTemplate, EnumVariantTemplate, FieldTemplate, StructTemplate, TypeTemplate,
};
use heck::ToPascalCase;
use std::{borrow::Cow, io};

pub mod template;

fn expand_node_name<'a>(
    schema: &'a cw_schema::SchemaV1,
    node: &'a cw_schema::Node,
) -> Cow<'a, str> {
    match node.value {
        cw_schema::NodeType::Array { items } => {
            let items = &schema.definitions[items];
            format!("[]{}", expand_node_name(schema, items)).into()
        }
        cw_schema::NodeType::Float => "float32".into(),
        cw_schema::NodeType::Double => "float64".into(),
        cw_schema::NodeType::Boolean => "bool".into(),
        cw_schema::NodeType::String => "string".into(),
        cw_schema::NodeType::Integer { signed, precision } => {
            let ty = if signed { "int" } else { "uint" };
            format!("{ty}{precision}").into()
        }
        cw_schema::NodeType::Binary => "[]byte".into(),
        cw_schema::NodeType::Optional { inner } => {
            let inner = &schema.definitions[inner];
            format!("{}", expand_node_name(schema, inner)).into()
        }
        cw_schema::NodeType::Struct(..) => node.name.as_ref().into(),
        cw_schema::NodeType::Tuple { ref items } => if items.len() == 1 {
            "interface{}"
        } else {
            "[]interface{}"
        }
        .into(),
        cw_schema::NodeType::Enum { .. } => node.name.as_ref().into(),

        cw_schema::NodeType::Decimal {
            precision: _,
            signed: _,
        } => {
            // ToDo: Actually use a decimal type here
            "string".into()
        }
        cw_schema::NodeType::Address => "Address".into(),
        cw_schema::NodeType::Checksum => "string".into(),
        cw_schema::NodeType::HexBinary => {
            // ToDo: Actually use a hex-encoded binary type here
            "string".into()
        }
        cw_schema::NodeType::Timestamp => "string".into(),
        cw_schema::NodeType::Unit => "struct{}".into(),
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
    add_package: bool,
) -> io::Result<()>
where
    O: io::Write,
{
    match node.value {
        cw_schema::NodeType::Struct(ref sty) => {
            let structt = StructTemplate {
                add_package,
                name: node.name.clone(),
                docs: prepare_docs(node.description.as_deref()),
                ty: match sty {
                    cw_schema::StructType::Unit => TypeTemplate::Unit,
                    cw_schema::StructType::Named { ref properties } => TypeTemplate::Named {
                        fields: properties
                            .iter()
                            .map(|(name, prop)| FieldTemplate {
                                name: Cow::Owned(name.to_pascal_case()),
                                rename: Cow::Borrowed(name),
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
                },
            };

            writeln!(output, "{structt}")?;
        }
        cw_schema::NodeType::Enum { ref cases, .. } => {
            let enumm = EnumTemplate {
                add_package,
                name: node.name.clone(),
                docs: prepare_docs(node.description.as_deref()),
                variants: cases
                    .iter()
                    .map(|(name, case)| EnumVariantTemplate {
                        name: name.to_pascal_case().into(),
                        rename: Cow::Borrowed(name),
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
                                            name: Cow::Owned(name.to_pascal_case()),
                                            rename: Cow::Borrowed(name),
                                            docs: prepare_docs(prop.description.as_deref()),
                                            ty: expand_node_name(
                                                schema,
                                                &schema.definitions[prop.value],
                                            ),
                                        })
                                        .collect(),
                                }
                            }
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
