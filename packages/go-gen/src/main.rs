use anyhow::{Context, Result};
use go::*;
use inflector::cases::pascalcase::to_pascal_case;
use schema::{documentation, schema_object_type, SchemaExt};
use schemars::schema::{ObjectValidation, RootSchema, Schema, SchemaObject};
use std::fmt::Write;

mod go;
mod schema;

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
    let (title, _) = schema::schema_object_type(&root.schema).context("failed to get type name")?;
    let main_type = build_type(&title, &root.schema)
        .with_context(|| format!("failed to generate {title}"))?
        .with_context(|| format!("failed to generate {title}, because it is a custom type"))?;

    let additional_types = root
        .definitions
        .into_iter()
        .filter_map(|(name, def)| {
            def.object()
                .map(|def| build_type(&name, def))
                .and_then(|r| r)
                .transpose()
        })
        .collect::<Result<Vec<_>>>()
        .context("failed to generate additional definitions")?;

    let mut code = format!("{main_type}\n");
    for additional_type in additional_types {
        writeln!(&mut code, "{additional_type}")?;
    }

    Ok(code)
}

fn build_type(name: &str, schema: &SchemaObject) -> Result<Option<GoStruct>> {
    if schema::custom_type_of(name).is_some() {
        // ignore custom types
        return Ok(None);
    }

    // first detect if we have a struct or enum
    if let Some(obj) = schema.object.as_ref() {
        build_struct(name, schema, obj)
            .map(Some)
            .with_context(|| format!("failed to generate struct '{name}'"))
    } else if let Some(variants) = schema::enum_variants(schema) {
        build_enum(name, schema, &variants)
            .map(Some)
            .with_context(|| format!("failed to generate enum '{name}'"))
    } else {
        // ignore other types
        Ok(None)
    }
}

pub fn build_struct(name: &str, strct: &SchemaObject, obj: &ObjectValidation) -> Result<GoStruct> {
    let docs = documentation(strct);

    // go through all fields
    let fields = obj.properties.iter().map(|(field, ty)| {
        // get schema object
        let ty = ty
            .object()
            .with_context(|| format!("expected schema object for field {field}"))?;
        // extract type from schema object
        let (go_type, is_nullable) = schema_object_type(ty)
            .with_context(|| format!("failed to get type of field '{field}'"))?;
        Ok(GoField {
            rust_name: field.clone(),
            docs: documentation(ty),
            ty: GoType {
                name: go_type,
                is_nullable,
            },
        })
    });
    let fields = fields.collect::<Result<Vec<_>>>()?;

    Ok(GoStruct {
        name: to_pascal_case(name),
        docs,
        fields,
    })
}

pub fn build_enum(name: &str, enm: &SchemaObject, variants: &[&Schema]) -> Result<GoStruct> {
    let docs = documentation(enm);

    // go through all fields
    let fields = variants.iter().map(|v| {
        // get schema object
        let v = v
            .object()
            .with_context(|| format!("expected schema object for enum variants of {name}"))?;

        // analyze the variant
        let (field, go_type) = schema::enum_variant(v).context("failed to extract enum variant")?;

        anyhow::Ok(GoField {
            rust_name: field,
            docs: documentation(v),
            ty: GoType {
                name: go_type,
                is_nullable: true, // always nullable
            },
        })
    });
    let fields = fields.collect::<Result<Vec<_>>>()?;

    Ok(GoStruct {
        name: name.to_string(),
        docs,
        fields,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Binary, HexBinary, Uint128};

    use super::*;

    fn assert_code_eq(actual: String, expected: &str) {
        let actual_no_ws = actual.split_whitespace().collect::<Vec<_>>();
        let expected_no_ws = expected.split_whitespace().collect::<Vec<_>>();

        assert!(
            actual_no_ws == expected_no_ws,
            "assertion failed: `(actual == expected)`\nactual:\n`{}`,\nexpected:\n`\"{}\"`",
            actual,
            expected
        );
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
    fn empty() {
        #[cw_serde]
        struct Empty {}

        let schema = schemars::schema_for!(Empty);
        let code = generate_go(schema).unwrap();
        assert_code_eq(code, "type Empty struct { }");
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

    #[test]
    fn nested_enum_works() {
        #[cw_serde]
        struct Inner {
            a: String,
        }

        #[cw_serde]
        enum MyEnum {
            A(Inner),
            B(String),
            C { a: String },
        }

        let schema = schemars::schema_for!(MyEnum);
        let code = generate_go(schema).unwrap();
        assert_code_eq(
            code,
            r#"
            type MyEnum struct {
                A *Inner `json:"a,omitempty"`
                B string `json:"b,omitempty"`
                C *CQuery `json:"c,omitempty"`
            }
            type Inner struct {
                A string `json:"a"`
            }
            type CQuery struct {
                A string `json:"a"`
            }
            "#,
        );
    }
}
