use std::collections::BTreeMap;

use schemars::{schema::RootSchema, JsonSchema};

pub use cosmwasm_schema_derive::QueryResponses;

pub trait QueryResponses: JsonSchema {
    fn response_schemas() -> BTreeMap<String, RootSchema> {
        let response_schemas = Self::response_schemas_impl();

        let queries: Vec<_> = response_schemas
            .keys()
            .map(std::borrow::Borrow::borrow)
            .collect();

        check_api_integrity::<Self>(&queries);

        response_schemas
    }

    fn response_schemas_impl() -> BTreeMap<String, RootSchema>;
}

/// `generated_queries` is expected to be a sorted slice here!
fn check_api_integrity<T: QueryResponses + ?Sized>(generated_queries: &[&str]) {
    let schema = crate::schema_for!(T);
    let mut schema_queries: Vec<_> = schema
        .schema
        .subschemas
        .unwrap()
        .one_of
        .unwrap()
        .into_iter()
        .map(|s| {
            s.into_object()
                .object
                .unwrap()
                .required
                .into_iter()
                .next()
                .unwrap()
        })
        .collect();
    schema_queries.sort();
    if schema_queries != generated_queries {
        // TODO: errors
        panic!("NOES");
    }
}
