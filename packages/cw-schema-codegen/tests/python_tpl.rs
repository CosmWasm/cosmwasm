use cw_schema::Schemaifier;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// This is a struct level documentation for enum type
#[derive(Schemaifier, Serialize, Deserialize, PartialEq, Debug)]
pub enum SomeEnum {
    /// Field1 docs
    Field1,

    /// Field2 docs
    Field2(u32, u32),

    /// Field3 docs
    Field3 {
        /// `a` field docs
        a: String,

        /// `b` field docs
        b: u32,
    },
}

/// This is a struct level documentation for unit struct
#[derive(Schemaifier, Serialize, Deserialize, PartialEq, Debug)]
pub struct UnitStructure;

/// This is a struct level documentation for tuple
#[derive(Schemaifier, Serialize, Deserialize, PartialEq, Debug)]
pub struct TupleStructure(u32, String, u128);

/// This is a struct level documentation for named structure
#[derive(Schemaifier, Serialize, Deserialize, PartialEq, Debug)]
pub struct NamedStructure {
    /// `a` field docs
    a: String,

    /// `b` field docs
    b: u8,

    /// `c` field docs
    c: SomeEnum,
}

#[derive(Schemaifier, Serialize, Deserialize, PartialEq, Debug)]
pub struct AllSimpleTypesAndDocs {
    array_field: Vec<String>,
    float_field: f32,
    double_field: f64,
    bool_field: bool,
    string_field: String,
    int_field: i64,
    bytes_field: cosmwasm_std::Binary,
    opt_field: Option<String>,
    byte_field: u8,
    decimal_field: cosmwasm_std::Decimal,
    address_field: cosmwasm_std::Addr,
    checksum_field: cosmwasm_std::Checksum,
    hexbinary_field: cosmwasm_std::HexBinary,
    timestamp_field: cosmwasm_std::Timestamp,
    unit_field: (),
}

#[test]
fn simple_enum() {
    // generate the schemas for each of the above types
    let schemas = [
        cw_schema::schema_of::<SomeEnum>(),
        cw_schema::schema_of::<UnitStructure>(),
        cw_schema::schema_of::<TupleStructure>(),
        cw_schema::schema_of::<NamedStructure>(),
    ];

    // run the codegen to typescript
    for schema in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            panic!();
        };

        let output = schema
            .definitions
            .iter()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::python::process_node(&mut buf, &schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        insta::assert_snapshot!(output);
    }
}

macro_rules! validator {
    ($ty:ty, $example:expr) => {{
        (
            ::std::any::type_name::<$ty>().replace("::", "_"),
            cw_schema::schema_of::<$ty>(),
            serde_json::to_string(&$example).unwrap(),
            {
                let a: Box<dyn FnOnce(&str)> = Box::new(|output| {
                    let result = serde_json::from_str::<$ty>(output).unwrap();
                    assert_eq!(result, $example);
                });
                a
            },
        )
    }};
}

#[test]
fn assert_validity() {
    let schemas = [
        validator!(SomeEnum, SomeEnum::Field1),
        validator!(SomeEnum, SomeEnum::Field2(10, 23)),
        validator!(
            SomeEnum,
            SomeEnum::Field3 {
                a: "sdf".to_string(),
                b: 12,
            }
        ),
        validator!(UnitStructure, UnitStructure {}),
        validator!(TupleStructure, TupleStructure(10, "aasdf".to_string(), 2)),
        validator!(
            NamedStructure,
            NamedStructure {
                a: "awer".to_string(),
                b: 4,
                c: SomeEnum::Field1,
            }
        ),
        validator!(
            AllSimpleTypesAndDocs,
            AllSimpleTypesAndDocs {
                array_field: vec!["abc".to_string(), "def".to_string()],
                float_field: 10.2,
                double_field: 10.232323,
                bool_field: true,
                string_field: "sdfsdf".to_string(),
                int_field: -10,
                bytes_field: cosmwasm_std::Binary::new(vec![0x1, 0x2, 0x3]),
                opt_field: Some("sdfsdfwer".to_string()),
                byte_field: 9,
                decimal_field: cosmwasm_std::Decimal::one(),
                address_field: cosmwasm_std::Addr::unchecked("some_address"),
                checksum_field: cosmwasm_std::Checksum::generate(&[0x10]),
                hexbinary_field: cosmwasm_std::HexBinary::from_hex("FAFAFA").unwrap(),
                timestamp_field: cosmwasm_std::Timestamp::from_seconds(100),
                unit_field: (),
            }
        ),
    ];

    for (type_name, schema, example, validator) in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            unreachable!();
        };

        let schema_output = schema
            .definitions
            .iter()
            .rev()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::python::process_node(&mut buf, &schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let mut file = tempfile::NamedTempFile::with_suffix(".py").unwrap();
        file.write_all(schema_output.as_bytes()).unwrap();
        file.write_all(
            format!(
                "import sys; print({type_name}.model_validate_json('{example}').model_dump_json())"
            )
            .as_bytes(),
        )
        .unwrap();
        file.flush().unwrap();

        let output = std::process::Command::new("python3")
            .arg(file.path())
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "stdout: {stdout}, stderr: {stderr}\n\n schema:\n {schema_output}",
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr),
        );

        validator(&String::from_utf8_lossy(&output.stdout))
    }
}
