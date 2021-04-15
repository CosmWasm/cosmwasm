#![cfg(feature = "stargate")]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StargateResponse {
    /// This is the protobuf response, binary encoded.
    /// The caller is responsible for knowing how to parse.
    pub response: Binary,
}
