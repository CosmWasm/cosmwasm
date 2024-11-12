use cw_schema::Schemaifier;

#[derive(Schemaifier)]
struct Owo {
    field_1: u32,
    field_2: String,
}

#[derive(Schemaifier)]
struct Uwu(String, u32);

#[derive(Schemaifier)]
struct Òwó;

#[derive(Schemaifier)]
enum Empty {}

#[derive(Schemaifier)]
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
