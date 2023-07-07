use cosmwasm_schema::cw_serde;

/// Replicates the cosmos-sdk bank module Metadata type
#[cw_serde]
#[derive(Eq, Default)]
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
#[cw_serde]
#[derive(Eq, Default)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    pub aliases: Vec<String>,
}
