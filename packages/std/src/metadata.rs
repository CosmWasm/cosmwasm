use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::prelude::*;

/// Replicates the cosmos-sdk bank module Metadata type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomMetadata {
    pub description: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}

/// Replicates the cosmos-sdk bank module DenomUnit type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub aliases: Vec<String>,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
