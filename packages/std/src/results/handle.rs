use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::encoding::Binary;
use crate::errors::StdResult;
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

pub type HandleResult<U = Empty> = StdResult<HandleResponse<U>>;

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
