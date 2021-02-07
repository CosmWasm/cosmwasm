use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::binary::Binary;
use crate::types::Empty;

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;
use super::mut_response::MutResponse;

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

impl<T> MutResponse<T> for MigrateResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn add_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
    }

    fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(msg.into());
    }

    fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}
