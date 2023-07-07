use cosmwasm_schema::cw_serde_prost;

/// Replicates the cosmos-sdk bank module Metadata type
#[cw_serde_prost]
#[derive(Eq)]
pub struct DenomMetadata {
    #[prost(string, tag = "1")]
    pub description: String,
    #[prost(message, repeated, tag = "2")]
    pub denom_units: Vec<DenomUnit>,
    #[prost(string, tag = "3")]
    pub base: String,
    #[prost(string, tag = "4")]
    pub display: String,
    #[prost(string, tag = "5")]
    pub name: String,
    #[prost(string, tag = "6")]
    pub symbol: String,
    #[prost(string, tag = "7")]
    pub uri: String,
    #[prost(string, tag = "8")]
    pub uri_hash: String,
}

/// Replicates the cosmos-sdk bank module DenomUnit type
#[cw_serde_prost]
#[derive(Eq)]
pub struct DenomUnit {
    #[prost(string, tag = "1")]
    pub denom: String,
    #[prost(uint32, tag = "2")]
    pub exponent: u32,
    #[prost(string, repeated, tag = "3")]
    pub aliases: Vec<String>,
}
