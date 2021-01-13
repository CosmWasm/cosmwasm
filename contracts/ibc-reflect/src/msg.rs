#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// InitMsg is a placeholder where we don't take any input
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub reflect_code_id: u64,
}

/// HandleMsg is a placeholder where we don't take any input
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleMsg {}
