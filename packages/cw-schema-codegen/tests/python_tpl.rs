use cw_schema::Schemaifier;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Schemaifier, Serialize, Deserialize)]
pub enum SomeEnum {
    Field1,
    Field2(u32, u32),
    Field3 { a: String, b: u32 },
    // Field4(Box<SomeEnum>),  // TODO tkulik: Do we want to support Box<T> ?
    // Field5 { a: Box<SomeEnum> },
}

#[derive(Schemaifier, Serialize, Deserialize)]
pub struct UnitStructure;

#[derive(Schemaifier, Serialize, Deserialize)]
pub struct TupleStructure(u32, String, u128);

#[derive(Schemaifier, Serialize, Deserialize)]
pub struct NamedStructure {
    a: String,
    b: u8,
    c: SomeEnum,
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
    ($typ:ty) => {{
        let a: Box<dyn FnOnce(&str) -> ()> = Box::new(|output| {
            serde_json::from_str::<$typ>(output).unwrap();
        });
        a
    }};
}

#[test]
fn assert_validity() {
    let schemas = [
        (
            "SomeEnum",
            cw_schema::schema_of::<SomeEnum>(),
            serde_json::to_string(&SomeEnum::Field1).unwrap(),
            validator!(SomeEnum),
        ),
        (
            "SomeEnum",
            cw_schema::schema_of::<SomeEnum>(),
            serde_json::to_string(&SomeEnum::Field2(10, 23)).unwrap(),
            validator!(SomeEnum),
        ),
        (
            "SomeEnum",
            cw_schema::schema_of::<SomeEnum>(),
            serde_json::to_string(&SomeEnum::Field3 {
                a: "sdf".to_string(),
                b: 12,
            })
            .unwrap(),
            validator!(SomeEnum),
        ),
        (
            "UnitStructure",
            cw_schema::schema_of::<UnitStructure>(),
            serde_json::to_string(&UnitStructure {}).unwrap(),
            validator!(UnitStructure),
        ),
        (
            "TupleStructure",
            cw_schema::schema_of::<TupleStructure>(),
            serde_json::to_string(&TupleStructure(10, "aasdf".to_string(), 2)).unwrap(),
            validator!(TupleStructure),
        ),
        (
            "NamedStructure",
            cw_schema::schema_of::<NamedStructure>(),
            serde_json::to_string(&NamedStructure {
                a: "awer".to_string(),
                b: 4,
                c: SomeEnum::Field1,
            })
            .unwrap(),
            validator!(NamedStructure),
        ),
    ];

    for (type_name, schema, example, validator) in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            unreachable!();
        };

        let schema_output = schema
            .definitions
            .iter()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::python::process_node(&mut buf, &schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let mut file = tempfile::NamedTempFile::with_suffix(".py").unwrap();
        file.write_all(schema_output.as_bytes()).unwrap();
        file.write(
            format!(
                "import sys; print({type_name}.model_validate_json('{example}').model_dump_json())"
            )
            .as_bytes(),
        )
        .unwrap();
        file.flush().unwrap();

        let output = std::process::Command::new("python")
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
