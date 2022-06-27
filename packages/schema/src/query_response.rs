use std::collections::BTreeMap;

use schemars::schema::RootSchema;

pub trait QueryResponses {
    fn query_responses() -> BTreeMap<&'static str, RootSchema>;
}
