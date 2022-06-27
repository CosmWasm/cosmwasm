use std::collections::BTreeMap;

use schemars::schema::RootSchema;

pub use cosmwasm_schema_derive::QueryResponses;

pub trait QueryResponses {
    fn query_responses() -> BTreeMap<&'static str, RootSchema>;
}
