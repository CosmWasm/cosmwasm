use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Replicates the cosmos-sdk bank module Metadata type
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

/// Replicates the cosmos-sdk bank module DenomUnit type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct NullableDenomMetadata {
    pub description: String,
    // denom_units is nullable: https://github.com/cosmos/cosmos-sdk/blob/main/api/cosmos/bank/v1beta1/bank.pulsar.go#L4539
    pub denom_units: Option<Vec<NullableDenomUnit>>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}

/// Replicates the cosmos-sdk bank module DenomUnit type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct NullableDenomUnit {
    pub denom: String,
    pub exponent: u32,
    // aliases is nullable: https://github.com/cosmos/cosmos-sdk/blob/main/api/cosmos/bank/v1beta1/bank.pulsar.go#L4478
    pub aliases: Option<Vec<String>>,
}
