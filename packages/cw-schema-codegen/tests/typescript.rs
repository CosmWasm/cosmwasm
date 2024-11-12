use cw_schema::Schemaifier;
use serde::Serialize;
use std::io::Write;

#[derive(Schemaifier, Serialize)]
struct Owo {
    field_1: u32,
    field_2: String,
}

#[derive(Schemaifier, Serialize)]
struct Uwu(String, u32);

#[derive(Schemaifier, Serialize)]
struct Òwó;

#[derive(Schemaifier, Serialize)]
enum Empty {}

#[derive(Schemaifier, Serialize)]
enum Hehehe {
    A,
    B(u32),
    C { field: String },
}

#[test]
fn codegen_snap() {
    // generate the schemas for each of the above types
    let schemas = [
        cw_schema::schema_of::<Owo>(),
        cw_schema::schema_of::<Uwu>(),
        cw_schema::schema_of::<Òwó>(),
        cw_schema::schema_of::<Empty>(),
        cw_schema::schema_of::<Hehehe>(),
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
                cw_schema_codegen::typescript::process_node(&mut buf, &schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        insta::assert_snapshot!(output);
    }
}

#[test]
fn assert_validity() {
    let schemas = [
        cw_schema::schema_of::<Owo>(),
        cw_schema::schema_of::<Uwu>(),
        cw_schema::schema_of::<Òwó>(),
        cw_schema::schema_of::<Empty>(),
        cw_schema::schema_of::<Hehehe>(),
    ];

    for schema in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            unreachable!();
        };

        let output = schema
            .definitions
            .iter()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::typescript::process_node(&mut buf, &schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let mut file = tempfile::NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(output.as_bytes()).unwrap();
        file.flush().unwrap();

        let output = std::process::Command::new("npx")
            .arg("--package=typescript")
            .arg("--")
            .arg("tsc")
            .arg(file.path())
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "stdout: {stdout}, stderr: {stderr}",
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr)
        );
    }
}
