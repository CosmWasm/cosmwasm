use anyhow::{bail, ensure, Context, Result};
use go::{GoField, GoType, GoTypeDef, GoTypeDefType};
use inflector::cases::pascalcase::to_pascal_case;
use schemars::schema::{InstanceType, RootSchema, SchemaObject, SingleOrVec};
use std::fmt::Write;

mod go;

fn main() -> Result<()> {
    let root = cosmwasm_schema::schema_for!(cosmwasm_std::AllDelegationsResponse);
    // println!(
    //     "{}",
    //     String::from_utf8(cosmwasm_std::to_vec(&root)?).unwrap()
    // );

    let code = generate_go(root)?;
    println!("{}", code);

    Ok(())
}

fn generate_go(root: RootSchema) -> Result<String> {
    let (title, _) = schema_object_type(&root.schema).context("failed to get type name")?;
    let main_type = generate_type(&title, root.schema)
        .with_context(|| format!("failed to generate {title}"))?
        .with_context(|| format!("failed to generate {title}, because it is a custom type"))?;

    let additional_types = root
        .definitions
        .into_iter()
        .filter_map(|(name, def)| generate_type(&name, def.into_object()).transpose())
        .collect::<Result<Vec<_>>>()
        .context("failed to generate additional definitions")?;

    let mut code = format!("{main_type}\n");
    for additional_type in additional_types {
        writeln!(&mut code, "{additional_type}")?;
    }

    Ok(code)
}

fn generate_type(name: &str, schema: SchemaObject) -> Result<Option<GoTypeDef>> {
    if custom_type_of(name).is_some() {
        // ignore custom types
        return Ok(None);
    }
    // first detect if we have a struct or enum
    if is_object(&schema) {
        generate_struct(name, schema).map(Some)
    } else if let Some(variants) = enum_variants(schema) {
        generate_enum(name, variants).map(Some)
    } else {
        // ignore other types
        Ok(None)
    }
}

fn generate_struct(name: &str, strct: SchemaObject) -> Result<GoTypeDef> {
    let docs = documentation(&strct);

    // go through all fields
    let o = strct
        .object
        .with_context(|| format!("failed to generate struct '{name}': expected object"))?;
    let fields = o.properties.into_iter().map(|(field, ty)| {
        let ty = ty.into_object();
        let (go_type, is_nullable) = schema_object_type(&ty)
            .with_context(|| format!("failed to get type of field '{field}' of struct '{name}'"))?;
        Ok(GoField {
            rust_name: field,
            docs: documentation(&ty),
            ty: GoType {
                name: go_type,
                is_nullable,
            },
        })
    });
    let fields = fields.collect::<Result<Vec<_>>>()?;

    Ok(GoTypeDef {
        name: to_pascal_case(name),
        docs,
        ty: GoTypeDefType::Struct { fields },
    })
}

fn generate_enum(name: &str, variants: Vec<SchemaObject>) -> Result<GoTypeDef> {
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

/// Returns the Go type for the given schema object and whether it is nullable.
fn schema_object_type(schema: &SchemaObject) -> Result<(String, bool)> {
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
        // if it has an instance type, use that
        if t.contains(&InstanceType::String) {
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
                match schema.array.as_ref().and_then(|a| a.items.as_ref()) {
                    Some(SingleOrVec::Single(array_validation)) => {
                        schema_object_type(&array_validation.clone().into_object())
                            .context("failed to get type of array item")?
                    }
                    _ => bail!("array type with non-singular item type not supported"),
                };
            // map custom types
            let item_type = custom_type_of(&item_type).unwrap_or(&item_type);

            if item_nullable {
                replace_custom_type(&format!("[]*{}", item_type))
            } else {
                replace_custom_type(&format!("[]{}", item_type))
            }
        } else {
            unreachable!("instance type should be one of the above")
        }
    } else if let Some(subschemas) = schema.subschemas.as_ref().and_then(|s| s.any_of.as_ref()) {
        // if one of them is null, use pointer type
        // TODO: ugly clone
        if let Some(null_index) = subschemas
            .iter()
            .position(|s| is_null(&s.clone().into_object()))
        {
            is_nullable = true;
            ensure!(subschemas.len() == 2, "multiple subschemas in anyOf");
            // extract non-null type
            let non_null_index = (null_index + 1) % 2;
            let (non_null_type, _) = schema_object_type(
                &subschemas
                    .get(non_null_index)
                    .expect("index should be valid")
                    .clone()
                    .into_object(),
            )?;
            // map custom types
            let non_null_type = custom_type_of(&non_null_type).unwrap_or(&non_null_type);
            non_null_type.to_string()
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
        let (ty, _) = schema_object_type(&subschema.clone().into_object())?;
        replace_custom_type(&ty)
    } else {
        bail!("no type found for schema: {:?}", schema);
    };

    Ok((ty, is_nullable))
}

fn is_null(schema: &SchemaObject) -> bool {
    schema
        .instance_type
        .as_ref()
        .map(|s| s.contains(&InstanceType::Null))
        .unwrap_or_default()
}

fn documentation(schema: &SchemaObject) -> Option<String> {
    schema
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_ref())
        .cloned()
}

/// Maps special types to their Go equivalents.
/// If the given type is not a special type, returns `None`.
fn custom_type_of(ty: &str) -> Option<&str> {
    match ty {
        "Uint128" => Some("string"),
        "Binary" => Some("[]byte"),
        "HexBinary" => Some("Checksum"),
        "Addr" => Some("string"),
        "Decimal" => Some("string"),
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
    use cosmwasm_std::{Binary, HexBinary, Uint128};
    use indenter::CodeFormatter;

    use super::*;

    fn assert_code_eq(actual: String, expected: &str) {
        let mut actual_fmt = String::new();
        let mut fmt = CodeFormatter::new(&mut actual_fmt, "    ");
        fmt.write_str(&actual).unwrap();

        let mut expected_fmt = String::new();
        let mut fmt = CodeFormatter::new(&mut expected_fmt, "    ");
        fmt.write_str(expected).unwrap();

        assert_eq!(actual_fmt, expected_fmt, "expected code to be equal");
    }

    #[test]
    fn special_types() {
        #[cw_serde]
        struct SpecialTypes {
            binary: Binary,
            nested_binary: Vec<Option<Binary>>,
            hex_binary: HexBinary,
            uint128: Uint128,
        }

        let schema = schemars::schema_for!(SpecialTypes);
        let code = generate_go(schema).unwrap();

        assert_code_eq(
            code,
            r#"
            type SpecialTypes struct {
                Binary []byte `json:"binary"`
                HexBinary Checksum `json:"hex_binary"`
                NestedBinary []*[]byte `json:"nested_binary"`
                Uint128 string `json:"uint128"`
            }"#,
        );
    }

    #[test]
    fn integers() {
        #[cw_serde]
        struct Integers {
            a: u64,
            b: i64,
            c: u32,
            d: i32,
            e: u8,
            f: i8,
            g: u16,
            h: i16,
        }

        let schema = schemars::schema_for!(Integers);
        let code = generate_go(schema).unwrap();

        assert_code_eq(
            code,
            r#"
            type Integers struct {
                A uint64 `json:"a"`
                B int64 `json:"b"`
                C uint32 `json:"c"`
                D int32 `json:"d"`
                E uint8 `json:"e"`
                F int8 `json:"f"`
                G uint16 `json:"g"`
                H int16 `json:"h"`
            }"#,
        );

        #[cw_serde]
        struct U128 {
            a: u128,
        }
        #[cw_serde]
        struct I128 {
            a: i128,
        }
        let schema = schemars::schema_for!(U128);
        assert!(generate_go(schema)
            .unwrap_err()
            .root_cause()
            .to_string()
            .contains("unsupported integer format: uint128"));
        let schema = schemars::schema_for!(I128);
        assert!(generate_go(schema)
            .unwrap_err()
            .root_cause()
            .to_string()
            .contains("unsupported integer format: int128"));
    }

    #[test]
    fn responses_work() {
        // bank
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
        // staking
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::BondedDenomResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::AllDelegationsResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::DelegationResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::AllValidatorsResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::ValidatorResponse
        ))
        .unwrap();
        // distribution
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::DelegatorWithdrawAddressResponse
        ))
        .unwrap();
        // wasm
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::ContractInfoResponse
        ))
        .unwrap();
        generate_go(cosmwasm_schema::schema_for!(cosmwasm_std::CodeInfoResponse)).unwrap();
    }
}
