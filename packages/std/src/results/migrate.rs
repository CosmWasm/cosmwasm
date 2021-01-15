use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::binary::Binary;
use crate::errors::StdError;
use crate::types::Empty;

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<T> Default for MigrateResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        MigrateResponse {
            messages: vec![],
            attributes: vec![],
            data: None,
        }
    }
}

#[deprecated(
    since = "0.12.1",
    note = "MigrateResult is deprecated because it uses StdError, which should be replaced with custom errors in CosmWasm 0.11+. \
            Replace this with Result<MigrateResponse, StdError> or Result<MigrateResponse<U>, StdError> and consider migrating to custom errors from there."
)]
pub type MigrateResult<U = Empty> = Result<MigrateResponse<U>, StdError>;
