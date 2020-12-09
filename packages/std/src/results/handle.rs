use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::binary::Binary;
use crate::errors::StdError;
use crate::types::Empty;

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<T> Default for HandleResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        HandleResponse {
            messages: vec![],
            attributes: vec![],
            data: None,
        }
    }
}

#[deprecated(
    since = "0.12.1",
    note = "HandleResult is deprecated because it uses StdError, which should be replaced with custom errors in CosmWasm 0.11+. \
            Replace this with Result<HandleResponse, StdError> or Result<HandleResponse<U>, StdError> and consider migrating to custom errors from there."
)]
pub type HandleResult<U = Empty> = Result<HandleResponse<U>, StdError>;
