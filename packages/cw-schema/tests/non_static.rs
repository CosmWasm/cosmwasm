#![allow(dead_code)]

use cw_schema::Schemaifier;
use std::borrow::Cow;

#[derive(Schemaifier)]
struct NonStatic<'a> {
    test1: &'a str,
    test2: Cow<'a, str>,
    test3: Cow<'static, str>,
    test4: &'static str,
}

#[test]
fn non_static_schema() {
    let schema = cw_schema::schema_of::<NonStatic<'_>>();
    insta::assert_json_snapshot!(schema);
}
