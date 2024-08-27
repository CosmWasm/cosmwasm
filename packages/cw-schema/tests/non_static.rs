#![allow(dead_code)]

use cw_schema::Schemaifier;

#[derive(Schemaifier)]
struct NonStatic<'a> {
    test: &'a str,
}
