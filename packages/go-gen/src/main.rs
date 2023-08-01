use anyhow::{bail, ensure, Context, Result};
use inflector::cases::pascalcase::to_pascal_case;
use schemars::schema::{InstanceType, RootSchema, SchemaObject, SingleOrVec};

fn main() -> Result<()> {
    let root = cosmwasm_schema::schema_for!(cosmwasm_std::SupplyResponse);
    println!("{:#?}", root);

    let code = generate_go(root)?;
    println!("{}", code);

    Ok(())
}

fn generate_go(root: RootSchema) -> Result<String> {
    let title = schema_object_type(&root.schema).context("failed to get type name")?;
    let mut code =
        generate_type(&title, root.schema).with_context(|| format!("error generating {title}"))?;
    let additional_types: String = root
        .definitions
        .into_iter()
        .map(|(name, def)| generate_type(&name, def.into_object()))
        .collect::<Result<Vec<_>>>()
        .context("failed generating additional definitions")? // TODO: better error message
        .join("\n");

    code.push('\n');
    code.push_str(&additional_types);

    Ok(code)
}

fn generate_type(name: &str, schema: SchemaObject) -> Result<String> {
    if custom_type_of(name).is_some() {
        // ignore custom types
        // TODO: ugly
        return Ok("".to_string());
    }
    // first detect if we have a struct or enum
    if is_object(&schema) {
        generate_struct(name, schema)
    } else if let Some(variants) = enum_variants(schema) {
        generate_enum(name, variants)
    } else {
        // ignore other types
        Ok("".to_string())
    }
}

fn generate_struct(name: &str, strct: SchemaObject) -> Result<String> {
    // generate documentation
    let mut out = String::new();
    if let Some(doc) = documentation(&strct, false) {
        out.push_str(&doc);
    }

    // type {name} struct {
    out.push_str("type ");
    out.push_str(name);
    out.push_str(" struct {\n");

    // go through all fields
    let o = strct.object.context("expected object")?;
    let fields = o
        .properties
        .into_iter()
        .map(|(field, ty)| (field, ty.into_object()));

    for (field, ty) in fields {
        if let Some(doc) = documentation(&ty, true) {
            out.push_str(&doc);
        }

        // {field} {type} `json:"{field}"`
        out.push_str("    ");
        out.push_str(&to_pascal_case(&field));
        out.push(' ');
        out.push_str(
            &schema_object_type(&ty)
                .with_context(|| format!("failed to get type of field {field}"))?,
        );
        out.push(' ');
        out.push_str("`json:\"");
        out.push_str(&field);
        out.push_str("\"`\n");
    }
    out.push('}');

    Ok(out)
}

fn generate_enum(name: &str, variants: Vec<SchemaObject>) -> Result<String> {
    todo!("generate_enum")
}

/// Returns `true` if the given schema is an object and `false` if it is not.
fn is_object(schema: &SchemaObject) -> bool {
    schema.object.is_some()
    // schema
    //     .instance_type
    //     .as_ref()
    //     .map(|s| {
    //         if let SingleOrVec::Single(s) = s {
    //             &InstanceType::Object == s.as_ref()
    //         } else {
    //             false
    //         }
    //     })
    //     .unwrap_or_default()
}

/// Returns the schemas of the variants of this enum, if it is an enum.
/// Returns `None` if the schema is not an enum.
fn enum_variants(schema: SchemaObject) -> Option<Vec<SchemaObject>> {
    Some(
        schema
            .subschemas?
            .one_of?
            .into_iter()
            .map(|s| s.into_object())
            .collect(),
    )
}

/// Returns the Go type for the given schema object.
fn schema_object_type(schema: &SchemaObject) -> Result<String> {
    // if it has a title, use that
    if let Some(title) = schema.metadata.as_ref().and_then(|m| m.title.as_ref()) {
        Ok(replace_custom_type(title))
    } else if let Some(reference) = &schema.reference {
        // if it has a reference, strip the path and use that
        Ok(replace_custom_type(
            reference
                .split('/')
                .last()
                .expect("split should always return at least one item"),
        ))
    } else if let Some(SingleOrVec::Single(t)) = &schema.instance_type {
        // if it has an instance type, use that
        let ty = match &**t {
            InstanceType::String => "string".to_string(),
            InstanceType::Number => "float64".to_string(),
            InstanceType::Integer => "int64".to_string(),
            InstanceType::Boolean => "bool".to_string(),
            InstanceType::Object => bail!("object type not supported: {:?}", schema),
            InstanceType::Array => {
                // get type of items
                let item_type = match schema.array.as_ref().and_then(|a| a.items.as_ref()) {
                    Some(SingleOrVec::Single(array_validation)) => {
                        schema_object_type(&array_validation.clone().into_object())?
                    }
                    _ => bail!("array type with non-singular item type not supported"),
                };
                // map custom types
                let item_type = custom_type_of(&item_type).unwrap_or(&item_type);

                format!("[]{}", item_type)
            }
            InstanceType::Null => bail!("null type not supported"),
        };

        Ok(ty)
    } else if let Some(subschemas) = schema.subschemas.as_ref().and_then(|s| s.any_of.as_ref()) {
        // if one of them is null, use pointer type
        // TODO: ugly clone
        if let Some(null_index) = subschemas
            .iter()
            .position(|s| is_null(&s.clone().into_object()))
        {
            ensure!(subschemas.len() == 2, "multiple subschemas in anyOf");
            // extract non-null type
            let non_null_index = (null_index + 1) % 2;
            let non_null_type = schema_object_type(
                &subschemas
                    .get(non_null_index)
                    .expect("index should be valid")
                    .clone()
                    .into_object(),
            )?;
            // map custom types
            let non_null_type = custom_type_of(&non_null_type).unwrap_or(&non_null_type);
            Ok(format!("*{non_null_type}"))
        } else if subschemas.len() == 1 {
            todo!("handle like allOf")
        } else {
            bail!("multiple anyOf without null type not supported")
        }
    } else if let Some(subschemas) = schema
        .subschemas
        .as_ref()
        .and_then(|s| s.all_of.as_ref().or(s.one_of.as_ref()))
    {
        ensure!(subschemas.len() == 1, "multiple subschemas in allOf");
        // just checked that there is exactly one subschema
        let subschema = subschemas.first().unwrap();

        // TODO: ugly clone
        Ok(replace_custom_type(&schema_object_type(
            &subschema.clone().into_object(),
        )?))
    } else {
        bail!("no type found for schema: {:?}", schema);
    }
}

fn is_null(schema: &SchemaObject) -> bool {
    schema
        .instance_type
        .as_ref()
        .map(|s| {
            if let SingleOrVec::Single(s) = s {
                InstanceType::Null == **s
            } else {
                false
            }
        })
        .unwrap_or_default()
}

fn documentation(schema: &SchemaObject, indented: bool) -> Option<String> {
    if let Some(description) = schema
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
    {
        // all new lines must be prefixed with `// `
        let replacement = if indented { "\n    // " } else { "\n// " };
        let docs = description.replace("\n", replacement);
        // and the first line too
        if indented {
            Some(format!("    // {}\n", docs))
        } else {
            Some(format!("// {}\n", docs))
        }
    } else {
        None
    }
}

/// Maps special types to their Go equivalents.
/// If the given type is not a special type, returns `None`.
fn custom_type_of(ty: &str) -> Option<&str> {
    match ty {
        "Uint128" => Some("string"),
        "Binary" => Some("[]byte"),
        _ => None,
    }
}

fn replace_custom_type(ty: &str) -> String {
    custom_type_of(ty)
        .map(|ty| ty.to_string())
        .unwrap_or_else(|| ty.to_string())
}

#[cfg(test)]
mod tests {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Binary, Uint128};

    use super::*;

    #[cw_serde]
    struct SpecialTypes {
        binary: Binary,
        nested_binary: Vec<Option<Binary>>,
        uint128: Uint128,
    }

    fn assert_code_eq(actual: String, expected: &str) {
        assert!(
            actual.split_whitespace().eq(expected.split_whitespace()),
            "expected code to be equal"
        );
    }

    #[test]
    fn special_types() {
        let schema = schemars::schema_for!(SpecialTypes);
        let code = generate_go(schema).unwrap();
        println!("{}", code);

        assert_code_eq(
            code,
            r#"
            type SpecialTypes struct {
                Binary []byte `json:"binary"`
                NestedBinary []*[]byte `json:"nested_binary"`
                Uint128 string `json:"uint128"`
            }"#,
        );
    }

    // TODO: write tests

    #[test]
    fn responses_work() {
        generate_go(cosmwasm_schema::schema_for!(cosmwasm_std::SupplyResponse)).unwrap();
        generate_go(cosmwasm_schema::schema_for!(cosmwasm_std::BalanceResponse)).unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::AllBalanceResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::DenomMetadataResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::AllDenomMetadataResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::ValidatorResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::AllValidatorsResponse
        ))
        .unwrap();
    }
}
