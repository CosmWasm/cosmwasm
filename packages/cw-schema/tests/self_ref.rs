#![allow(dead_code)]

#[derive(cw_schema::Schemaifier)]
pub struct SelfRef {
    meow: Vec<SelfRef>,
}

#[test]
fn selfref() {
    insta::assert_json_snapshot!(cw_schema::schema_of::<SelfRef>());
}
