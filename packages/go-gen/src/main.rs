use anyhow::{bail, ensure, Context, Result};
use go::*;
use inflector::cases::pascalcase::to_pascal_case;
use schema::{documentation, schema_object_type, SchemaExt, TypeContext};
use schemars::schema::{ObjectValidation, RootSchema, Schema, SchemaObject};
use std::fmt::Write;
use utils::replace_acronyms;

mod go;
mod schema;
mod utils;

fn main() -> Result<()> {
    let root = cosmwasm_schema::schema_for!(cosmwasm_std::BankQuery);

    let code = generate_go(root)?;
    println!("{}", code);

    Ok(())
}

/// Generates the Go code for the given schema
fn generate_go(root: RootSchema) -> Result<String> {
    let title = replace_acronyms(
        root.schema
            .metadata
            .as_ref()
            .and_then(|m| m.title.as_ref())
            .context("failed to get type name")?,
    );

    let mut types = vec![];
    build_type(&title, &root.schema, &mut types)
        .with_context(|| format!("failed to generate {title}"))?;

    // go through additional definitions
    for (name, additional_type) in &root.definitions {
        additional_type
            .object()
            .map(|def| build_type(&replace_acronyms(name), def, &mut types))
            .and_then(|r| r)
            .context("failed to generate additional definitions")?;
    }
    let mut code = String::new();
    for ty in types {
        writeln!(&mut code, "{ty}")?;
    }

    Ok(code)
}

/// Generates Go structs for the given schema and adds them to `structs`.
/// This will add more than one struct if the schema contains object types (anonymous structs).
fn build_type(name: &str, schema: &SchemaObject, structs: &mut Vec<GoStruct>) -> Result<()> {
    if schema::custom_type_of(name).is_some() {
        // ignore custom types
        return Ok(());
    }

    // first detect if we have a struct or enum
    if let Some(obj) = schema.object.as_ref() {
        let strct = build_struct(name, schema, obj, structs)
            .map(Some)
            .with_context(|| format!("failed to generate struct '{name}'"))?;
        if let Some(strct) = strct {
            structs.push(strct);
        }
    } else if let Some(variants) = schema::enum_variants(schema) {
        let strct = build_enum(name, schema, variants, structs)
            .map(Some)
            .with_context(|| format!("failed to generate enum '{name}'"))?;
        if let Some(strct) = strct {
            structs.push(strct);
        }
    } else {
        anyhow::bail!("failed to determine type for '{name}'");
    }

    Ok(())
}

/// Creates a Go struct for the given schema object and returns it.
/// This will also add any additional structs to `additional_structs` (but not the returned one).
pub fn build_struct(
    name: &str,
    strct: &SchemaObject,
    obj: &ObjectValidation,
    additional_structs: &mut Vec<GoStruct>,
) -> Result<GoStruct> {
    let docs = documentation(strct);

    // go through all fields
    let fields = obj.properties.iter().map(|(field, ty)| {
        // get schema object
        let schema = ty
            .object()
            .with_context(|| format!("expected schema object for field {field}"))?;
        // extract type from schema object
        let ty = schema_object_type(schema, TypeContext::new(name, field), additional_structs)
            .with_context(|| format!("failed to get type of field '{field}'"))?;
        Ok(GoField {
            rust_name: field.clone(),
            docs: documentation(schema),
            ty,
        })
    });
    let fields = fields.collect::<Result<Vec<_>>>()?;

    Ok(GoStruct {
        name: to_pascal_case(name),
        docs,
        fields,
    })
}

/// Creates a Go struct for the given schema object and returns it.
/// This will also add any additional structs to `additional_structs` (but not the returned one).
pub fn build_enum<'a>(
    name: &str,
    enm: &SchemaObject,
    variants: impl Iterator<Item = &'a Schema>,
    additional_structs: &mut Vec<GoStruct>,
) -> Result<GoStruct> {
    let docs = documentation(enm);

    // go through all fields
    let fields = variants.map(|v| {
        // get schema object
        let v = v
            .object()
            .with_context(|| format!("expected schema object for enum variants of {name}"))?;

        // analyze the variant
        let variant_field = build_enum_variant(v, name, additional_structs)
            .context("failed to extract enum variant")?;

        anyhow::Ok(variant_field)
    });
    let fields = fields.collect::<Result<Vec<_>>>()?;

    Ok(GoStruct {
        name: name.to_string(),
        docs,
        fields,
    })
}

/// Tries to extract the name and type of the given enum variant and returns it as a `GoField`.
pub fn build_enum_variant(
    schema: &SchemaObject,
    enum_name: &str,
    additional_structs: &mut Vec<GoStruct>,
) -> Result<GoField> {
    // for variants without inner data, there is an entry in `enum_variants`
    // we are not interested in that case, so we error out
    if let Some(values) = &schema.enum_values {
        bail!(
            "enum variants {} without inner data not supported",
            values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let docs = documentation(schema);

    // for variants with inner data, there is an object validation entry with a single property
    // we extract the type of that property
    let properties = &schema
        .object
        .as_ref()
        .context("expected object validation for enum variant")?
        .properties;
    ensure!(
        properties.len() == 1,
        "expected exactly one property in enum variant"
    );
    // we can unwrap here, because we checked the length above
    let (name, schema) = properties.first_key_value().unwrap();
    let GoType { name: ty, .. } = schema_object_type(
        schema.object()?,
        TypeContext::new(enum_name, name),
        additional_structs,
    )?;

    Ok(GoField {
        rust_name: name.to_string(),
        docs,
        ty: GoType {
            name: ty,
            is_nullable: true, // always nullable
        },
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Binary, Empty, HexBinary, Uint128};

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

    fn assert_code_eq_ignore_docs(actual: String, expected: &str) {
        let actual_filtered = actual
            .lines()
            .map(|line| line.split("//").next().unwrap()) // ignore comments
            .flat_map(|line| line.split_whitespace())
            .collect::<Vec<_>>();
        let expected_filtered = expected
            .lines()
            .map(|line| line.split("//").next().unwrap()) // ignore comments
            .flat_map(|line| line.split_whitespace())
            .collect::<Vec<_>>();

        assert!(
            actual_filtered == expected_filtered,
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

    /// Compares the generated code for a given type with the code in the corresponding file in
    /// `tests/`.
    /// The file name is derived from the type name by replacing `::` with `__` and adding `.go`.
    macro_rules! compare_codes {
        ($name:ty) => {{
            let filename = stringify!($name).replace("::", "__");
            let generated = generate_go(cosmwasm_schema::schema_for!($name)).unwrap();
            let expected = std::fs::read_to_string(format!("tests/{}.go", filename)).unwrap();

            assert_code_eq_ignore_docs(generated, &expected);
        }};
    }

    #[test]
    fn responses_work() {
        // bank
        compare_codes!(cosmwasm_std::SupplyResponse);
        compare_codes!(cosmwasm_std::BalanceResponse);
        // compare_codes!(cosmwasm_std::AllBalanceResponse); // has different name in wasmvm
        compare_codes!(cosmwasm_std::DenomMetadataResponse);
        // compare_codes!(cosmwasm_std::AllDenomMetadataResponse); // uses `[]byte` instead of `*[]byte`
        // staking
        compare_codes!(cosmwasm_std::BondedDenomResponse);
        compare_codes!(cosmwasm_std::AllDelegationsResponse);
        compare_codes!(cosmwasm_std::DelegationResponse);
        compare_codes!(cosmwasm_std::AllValidatorsResponse);
        // compare_codes!(cosmwasm_std::ValidatorResponse); // does not use "omitempty" for `Validator` field
        // distribution
        compare_codes!(cosmwasm_std::DelegatorWithdrawAddressResponse);
        // wasm
        compare_codes!(cosmwasm_std::ContractInfoResponse);
        // compare_codes!(cosmwasm_std::CodeInfoResponse); // TODO: Checksum type and "omitempty"
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
            type CEnum struct {
                A string `json:"a"`
            }
            type MyEnum struct {
                A *Inner `json:"a,omitempty"`
                B string `json:"b,omitempty"`
                C *CEnum `json:"c,omitempty"`
            }
            type Inner struct {
                A string `json:"a"`
            }
            "#,
        );

        #[cw_serde]
        enum ShouldFail1 {
            A(),
        }
        #[cw_serde]
        enum ShouldFail2 {
            A,
        }
        let schema = schemars::schema_for!(ShouldFail1);
        assert!(generate_go(schema)
            .unwrap_err()
            .root_cause()
            .to_string()
            .contains("array type with non-singular item type is not supported"));
        let schema = schemars::schema_for!(ShouldFail2);
        assert!(generate_go(schema)
            .unwrap_err()
            .root_cause()
            .to_string()
            .contains("failed to determine type for 'ShouldFail2'"));
    }

    #[test]
    fn queries_work() {
        // compare_codes!(cosmwasm_std::QueryRequest<Empty>); // omit for now because it's huge
        // just assert that it compiles
        generate_go(cosmwasm_schema::schema_for!(
            cosmwasm_std::QueryRequest<Empty>
        ))
        .unwrap();
        // TODO: PageRequest.Key uses "omitempty" and no *
        // compare_codes!(cosmwasm_std::BankQuery);
        compare_codes!(cosmwasm_std::StakingQuery);
        compare_codes!(cosmwasm_std::DistributionQuery);
        compare_codes!(cosmwasm_std::IbcQuery);
        compare_codes!(cosmwasm_std::WasmQuery);
    }

    #[test]
    fn array_item_type_works() {
        #[cw_serde]
        struct A {
            a: Vec<Vec<Vec<Option<Option<B>>>>>,
        }
        #[cw_serde]
        struct B {}

        // example json:
        // A { a: vec![vec![vec![None, Some(Some(B {})), Some(None)]]] }
        // => {"a":[[[null,{},null]]]}
        let code = generate_go(cosmwasm_schema::schema_for!(A)).unwrap();
        assert_code_eq(
            code,
            r#"
            type A struct {
                A [][][]*B `json:"a"`
            }
            type B struct { }"#,
        );

        #[cw_serde]
        struct C {
            c: Vec<Vec<Vec<Option<Option<String>>>>>,
        }
        let code = generate_go(cosmwasm_schema::schema_for!(C)).unwrap();
        assert_code_eq(
            code,
            r#"
            type C struct {
                C [][][]*string `json:"c"`
            }"#,
        );
    }
}
