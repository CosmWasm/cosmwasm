use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Uint128;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomMetadata {
    pub description: String,
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: Uint128,
    pub aliases: Vec<String>,
}
