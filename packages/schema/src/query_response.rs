use std::collections::BTreeMap;

use schemars::{schema::RootSchema, JsonSchema};

pub use cosmwasm_schema_derive::QueryResponses;

pub trait QueryResponses: JsonSchema {
    fn response_schemas() -> BTreeMap<String, RootSchema>;

    /// `generated_queries` is expected to be a sorted slice here!
    fn check_api_integrity(generated_queries: &[&str]) {
        let schema = crate::schema_for!(Self);
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
}
