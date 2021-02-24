use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, ContractResult};

use super::{Attribute, CosmosMsg, Empty};

/// A sub-message that will guarantee a subcall_response callback on success or error
/// Note on error the subcall will revert any partial state changes due to this message,
/// but not revert any state changes in the calling contract (that must be done in the
/// subcall_response entry point)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub id: u64,
    pub msg: CosmosMsg<T>,
    pub gas_limit: Option<u64>,
}

/// The Result object returned to subcall_response. We always get the same id back
/// and then must handle success and error cases ourselves
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubCallResult {
    pub id: u64,
    pub result: ContractResult<SubCallResponse>,
}

/// The information we get back from a successful sub-call, with full sdk events
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubCallResponse {
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

/// a full sdk event
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: Vec<Attribute>,
}

impl Event {
    pub fn new(kind: &str, attributes: Vec<Attribute>) -> Self {
        Event {
            kind: kind.to_string(),
            attributes,
        }
    }
}
