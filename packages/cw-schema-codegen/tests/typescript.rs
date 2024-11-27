use arbitrary::Arbitrary;
use core::str;
use cw_schema::Schemaifier;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

#[derive(Arbitrary, Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
struct Owo {
    field_1: u32,
    field_2: String,
}

#[derive(Arbitrary, Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
struct Uwu(String, u32);

#[derive(Arbitrary, Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
struct Òwó;

#[derive(Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
pub enum Empty {}

#[derive(Arbitrary, Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
enum Hehehe {
    A,
    B(u32),
    C { field: String },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
enum Combined {
    Owo(Owo),
    Uwu(Uwu),
    Òwó(Òwó),
    Empty(Empty),
    Hehehe(Hehehe),
}

macro_rules! impl_from {
    ($ty:ident) => {
        impl From<$ty> for Combined {
            fn from(ty: $ty) -> Combined {
                Combined::$ty(ty)
            }
        }
    };
}

impl_from!(Owo);
impl_from!(Uwu);
impl_from!(Òwó);
impl_from!(Empty);
impl_from!(Hehehe);

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
                cw_schema_codegen::typescript::process_node(&mut buf, &schema, node, true).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        insta::assert_snapshot!(output);
    }
}

fn wrap<T: for<'a> Arbitrary<'a> + Into<Combined>>(
    stuff: &mut arbitrary::Unstructured,
) -> Combined {
    T::arbitrary(stuff).unwrap().into()
}

fn type_name<T>() -> String {
    let name = std::any::type_name::<T>().split(':').last().unwrap();
    format!("{name}Schema")
}

#[test]
fn assert_validity() {
    #[allow(clippy::type_complexity)]
    let schemas: &[(_, fn(&mut arbitrary::Unstructured) -> Combined, _)] = &[
        (
            cw_schema::schema_of::<Owo>(),
            wrap::<Owo>,
            type_name::<Owo>(),
        ),
        (
            cw_schema::schema_of::<Uwu>(),
            wrap::<Uwu>,
            type_name::<Uwu>(),
        ),
        (
            cw_schema::schema_of::<Òwó>(),
            wrap::<Òwó>,
            type_name::<Òwó>(),
        ),
        // `Empty` is a non-constructable type
        /*(
            cw_schema::schema_of::<Empty>(),
            wrap::<Empty>,
            type_name::<Empty>(),
        ),*/
        (
            cw_schema::schema_of::<Hehehe>(),
            wrap::<Hehehe>,
            type_name::<Hehehe>(),
        ),
    ];

    let e2e_dir = format!("{}/tests/ts-e2e", env!("CARGO_MANIFEST_DIR"));
    let gen_file_path = format!("{}/src/gen.ts", e2e_dir);

    // make sure the dependencies are installed
    let install_status = Command::new("npm")
            .arg("i")
            .current_dir(&e2e_dir)
            .status()
            .unwrap();
        assert!(install_status.success());

    let random_data: [u8; 255] = rand::random();
    let mut unstructured = arbitrary::Unstructured::new(&random_data);
    for (schema, arbitrary_gen, type_name) in schemas {
        let cw_schema::Schema::V1(schema) = schema else {
            unreachable!();
        };

        let output = schema
            .definitions
            .iter()
            .map(|node| {
                let mut buf = Vec::new();
                cw_schema_codegen::typescript::process_node(&mut buf, schema, node, true).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let mut gen_file = File::create(&gen_file_path).unwrap();
        gen_file.write_all(output.as_bytes()).unwrap();

        let data = arbitrary_gen(&mut unstructured);
        let serialized = serde_json::to_string(&data).unwrap();

        let mut child = Command::new("npm")
            .args(["test", type_name])
            .current_dir(&e2e_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(serialized.as_bytes()).unwrap();
        }

        let proc_output = child.wait_with_output().unwrap();
        assert!(
            proc_output.status.success(),
            "failed with object: {data:#?}; json: {serialized}; schema: {output}"
        );

        let stdout = str::from_utf8(&proc_output.stdout).unwrap();
        let stdout = stdout.lines().last().unwrap();
        let deserialized: Combined = serde_json::from_str(stdout).unwrap_or_else(|err| {
            panic!("{err:?}; input: {serialized}, output: {stdout}");
        });

        assert_eq!(data, deserialized);
    }
}
