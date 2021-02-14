#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use cosmwasm_std::{Binary, Deps, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Cosmos format (secp256k1 verification scheme).
    VerifyCosmosSignature {
        /// Message to verify.
        message: Binary,
        /// Serialized signature. Cosmos format (64 bytes).
        signature: Binary,
        /// Serialized compressed (33 bytes) or uncompressed (65 bytes) public key.
        public_key: Binary,
    },
    /// Tendermint format (ed25519 verification scheme).
    VerifyTendermintSignature {
        /// Message to verify.
        message: Binary,
        /// Serialized signature. Tendermint format (64 bytes).
        signature: Binary,
        /// Serialized public key. Tendermint format (32 bytes).
        public_key: Binary,
    },
    /// Returns a list of supported verification schemes.
    /// No pagination - this is a short list.
    ListVerificationSchemes {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VerifyResponse {
    pub verifies: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListVerificationsResponse {
    pub verification_schemes: Vec<String>,
}

pub(crate) fn list_verifications(_deps: Deps) -> StdResult<Vec<String>> {
    Ok(vec!["secp256k1".into(), "ed25519".into()])
}
