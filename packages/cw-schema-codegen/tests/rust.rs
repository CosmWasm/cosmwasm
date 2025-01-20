use arbitrary::Arbitrary;
use cw_schema::Schemaifier;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
    str,
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

fn wrap<T: for<'a> Arbitrary<'a> + Into<Combined>>(
    stuff: &mut arbitrary::Unstructured,
) -> Combined {
    T::arbitrary(stuff).unwrap().into()
}

fn type_name<T>() -> String {
    std::any::type_name::<T>()
        .replace("::", "_")
        .replace(['<', '>'], "_")
}

#[test]
fn e2e() {
    #[allow(clippy::type_complexity)]
    let schemas: &[(_, fn(&mut arbitrary::Unstructured<'_>) -> Combined, _)] = &[
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

    let e2e_dir = format!("{}/tests/rust-e2e", env!("CARGO_MANIFEST_DIR"));
    let gen_file_path = format!("{e2e_dir}/src/gen.rs");

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
                cw_schema_codegen::rust::process_node(&mut buf, schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let mut gen_file = File::create(&gen_file_path).unwrap();
        gen_file.write_all(output.as_bytes()).unwrap();
        gen_file
            .write_all(format!("pub type TestType = {type_name};").as_bytes())
            .unwrap();

        let data = arbitrary_gen(&mut unstructured);
        let serialized = serde_json::to_string(&data).unwrap();

        let mut child = Command::new("cargo")
            .args(["run"])
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
