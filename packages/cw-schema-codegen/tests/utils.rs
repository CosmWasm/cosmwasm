use std::{
    collections::BTreeMap,
    io::{self, Write},
    process::{Command, Stdio},
};

use arbitrary::Arbitrary;
use cw_schema::Schemaifier;
use serde::{Deserialize, Serialize};

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

#[derive(Arbitrary, Schemaifier, Debug, Deserialize, PartialEq, Serialize)]
struct Foo {
    // foo_field_0: f32, // can cause rounding errors
    // foo_field_1: f64,
    foo_field_2: bool,
    foo_field_3: String,
    // foo_field_4: i128, // not supported on all platforms (e.g. Go)
    foo_field_4: i32,
    foo_field_5: u8,
    foo_field_6: Vec<String>,
    foo_field_7: Option<String>,
    foo_field_9: Box<str>,
    foo_field_10: BTreeMap<String, u32>,
    foo_field_11: (u32, String),
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
enum Combined {
    Owo(Owo),
    Uwu(Uwu),
    Òwó(Òwó),
    Empty(Empty),
    Hehehe(Hehehe),
    Foo(Foo),
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
impl_from!(Foo);

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

pub struct TestCase<'a> {
    pub code: &'a str,
    pub type_name: &'a str,
}

pub fn run_e2e(
    process_node: impl Fn(&mut Vec<u8>, &cw_schema::SchemaV1, &cw_schema::Node) -> io::Result<()>,
    mut run_code: impl FnMut(TestCase) -> Command,
) {
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
        (
            cw_schema::schema_of::<Foo>(),
            wrap::<Foo>,
            type_name::<Foo>(),
        ),
    ];

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
                process_node(&mut buf, schema, node).unwrap();
                String::from_utf8(buf).unwrap()
            })
            .collect::<String>();

        let data = arbitrary_gen(&mut unstructured);
        let serialized = serde_json::to_string(&data).unwrap();
        let mut child = run_code(TestCase {
            code: &output,
            type_name,
        })
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

        let stdout = core::str::from_utf8(&proc_output.stdout).unwrap();
        let stdout = stdout.lines().last().unwrap();
        let deserialized: Combined = serde_json::from_str(stdout).unwrap_or_else(|err| {
            panic!("{err:?}; input: {serialized}, output: {stdout}");
        });

        assert_eq!(data, deserialized);
    }
}
