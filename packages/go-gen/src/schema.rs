use anyhow::{bail, ensure, Context, Result};

use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};

pub trait SchemaExt {
    fn object(&self) -> anyhow::Result<&SchemaObject>;
}

impl SchemaExt for Schema {
    fn object(&self) -> anyhow::Result<&SchemaObject> {
        match self {
            Schema::Object(o) => Ok(o),
            _ => bail!("expected schema object"),
        }
    }
}

/// Returns the schemas of the variants of this enum, if it is an enum.
/// Returns `None` if the schema is not an enum.
pub(crate) fn enum_variants(schema: &SchemaObject) -> Option<Vec<&Schema>> {
    Some(
        schema
            .subschemas
            .as_ref()?
            .one_of
            .as_ref()?
            .iter()
            .collect(),
    )
}

/// Returns the Go type for the given schema object and whether it is nullable.
pub(crate) fn schema_object_type(schema: &SchemaObject) -> Result<(String, bool)> {
    let mut is_nullable = is_null(schema);

    // if it has a title, use that
    let ty = if let Some(title) = schema.metadata.as_ref().and_then(|m| m.title.as_ref()) {
        replace_custom_type(title)
    } else if let Some(reference) = &schema.reference {
        // if it has a reference, strip the path and use that
        replace_custom_type(
            reference
                .split('/')
                .last()
                .expect("split should always return at least one item"),
        )
    } else if let Some(t) = &schema.instance_type {
        type_from_instance_type(schema, t)?
    } else if let Some(subschemas) = schema.subschemas.as_ref().and_then(|s| s.any_of.as_ref()) {
        // check if one of them is null
        let nullable = nullable_type(subschemas)?;
        if let Some(non_null) = nullable {
            ensure!(subschemas.len() == 2, "multiple subschemas in anyOf");
            is_nullable = true;
            // extract non-null type
            let (non_null_type, _) = schema_object_type(non_null)?;
            replace_custom_type(&non_null_type)
        } else {
            subschema_type(subschemas).context("failed to get type of anyOf subschemas")?
        }
    } else if let Some(subschemas) = schema
        .subschemas
        .as_ref()
        .and_then(|s| s.all_of.as_ref().or(s.one_of.as_ref()))
    {
        subschema_type(subschemas).context("failed to get type of allOf subschemas")?
    } else {
        bail!("no type for schema found: {:?}", schema);
    };

    Ok((ty, is_nullable))
}

/// Tries to extract the type of the non-null variant of an anyOf schema.
///
/// Returns `Ok(None)` if the type is not nullable.
pub(crate) fn nullable_type(subschemas: &[Schema]) -> Result<Option<&SchemaObject>, anyhow::Error> {
    let (found_null, nullable_type): (bool, Option<&SchemaObject>) = subschemas
        .iter()
        .fold(Ok((false, None)), |result: Result<_>, subschema| {
            result.and_then(|(nullable, not_null)| {
                let subschema = subschema.object()?;
                if is_null(subschema) {
                    Ok((true, not_null))
                } else {
                    Ok((nullable, Some(subschema)))
                }
            })
        })
        .context("failed to get anyOf subschemas")?;

    Ok(if found_null { nullable_type } else { None })
}

/// Tries to extract a type name from the given instance type.
///
/// Fails for unsupported instance types or integer formats.
pub(crate) fn type_from_instance_type(
    schema: &SchemaObject,
    t: &SingleOrVec<InstanceType>,
) -> Result<String> {
    // if it has an instance type, use that
    Ok(if t.contains(&InstanceType::String) {
        "string".to_string()
    } else if t.contains(&InstanceType::Number) {
        "float64".to_string()
    } else if t.contains(&InstanceType::Integer) {
        const AVAILABLE_INTS: &[&str] = &[
            "uint8", "int8", "uint16", "int16", "uint32", "int32", "uint64", "int64",
        ];
        let format = schema.format.as_deref().unwrap_or("int64");
        if AVAILABLE_INTS.contains(&format) {
            format.to_string()
        } else {
            bail!("unsupported integer format: {}", format);
        }
    } else if t.contains(&InstanceType::Boolean) {
        "bool".to_string()
    } else if t.contains(&InstanceType::Object) {
        bail!("object type not supported: {:?}", schema);
    } else if t.contains(&InstanceType::Array) {
        // get type of items
        let (item_type, item_nullable) =
            array_item_type(schema).context("failed to get array item type")?;
        // map custom types
        let item_type = custom_type_of(&item_type).unwrap_or(&item_type);

        if item_nullable {
            replace_custom_type(&format!("[]*{}", item_type))
        } else {
            replace_custom_type(&format!("[]{}", item_type))
        }
    } else {
        unreachable!("instance type should be one of the above")
    })
}

/// Extract the type of the items of an array.
///
/// This fails if the given schema object is not an array,
/// has multiple item types or other errors occur during type extraction of
/// the underlying schema.
pub(crate) fn array_item_type(schema: &SchemaObject) -> Result<(String, bool)> {
    match schema.array.as_ref().and_then(|a| a.items.as_ref()) {
        Some(SingleOrVec::Single(array_validation)) => {
            schema_object_type(array_validation.object()?)
        }
        _ => bail!("array type with non-singular item type not supported"),
    }
}

/// Tries to extract a type name from the given subschemas.
///
/// This fails if there are multiple subschemas or other errors occur
/// during subschema type extraction.
pub(crate) fn subschema_type(subschemas: &[Schema]) -> Result<String> {
    ensure!(
        subschemas.len() == 1,
        "multiple subschemas are not supported"
    );
    let subschema = &subschemas[0];
    let (ty, _) = schema_object_type(subschema.object()?)?;
    Ok(replace_custom_type(&ty))
}

pub(crate) fn is_null(schema: &SchemaObject) -> bool {
    schema
        .instance_type
        .as_ref()
        .map_or(false, |s| s.contains(&InstanceType::Null))
}

pub(crate) fn documentation(schema: &SchemaObject) -> Option<String> {
    schema.metadata.as_ref()?.description.as_ref().cloned()
}

/// Maps special types to their Go equivalents.
/// If the given type is not a special type, returns `None`.
pub(crate) fn custom_type_of(ty: &str) -> Option<&str> {
    match ty {
        "Uint128" => Some("string"),
        "Binary" => Some("[]byte"),
        "HexBinary" => Some("Checksum"),
        "Addr" => Some("string"),
        "Decimal" => Some("string"),
        _ => None,
    }
}

pub(crate) fn replace_custom_type(ty: &str) -> String {
    custom_type_of(ty)
        .map(|ty| ty.to_string())
        .unwrap_or_else(|| ty.to_string())
}
