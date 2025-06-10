use cw_schema::Schema;

#[test]
fn can_decode_example() {
    let example = include_str!("example.json");
    let _: Schema = serde_json::from_str(example).unwrap();
}

#[test]
fn snapshot_jsonschema() {
    let schema = schemars::schema_for!(Schema);
    insta::assert_json_snapshot!(schema);
}
