#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use cosmwasm_std::{Binary, Deps, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    VerifySignature {
        /// Sha-256 hash of the message to verify (32 bytes).
        message_hash: Binary,
        /// Serialized signature. Cosmos format (64 bytes).
        signature: Binary,
        /// Serialized compressed (33 bytes) or uncompressed (65 bytes) public key.
        public_key: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a list of supported verification schemes.
    /// No pagination - this is a short list.
    ListVerificationSchemes {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListVerificationsResponse {
    pub verification_schemes: Vec<String>,
}

pub fn list_verifications(_deps: Deps) -> StdResult<Vec<String>> {
    Ok(vec!["secp256k1".into()])
}
