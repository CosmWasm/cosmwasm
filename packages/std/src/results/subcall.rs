use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, ContractResult};

use super::{Attribute, CosmosMsg, Empty};

/// Use this to define when the contract gets a response callback.
/// If you only need it for errors or success you can select just those in order
/// to save gas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ReplyOn {
    /// Always perform a callback after SubMsg is processed
    Always,
    /// Only callback if SubMsg returned an error, no callback on success case
    Error,
    /// Only callback if SubMsg was successful, no callback on error case
    Success,
}

impl Default for ReplyOn {
    fn default() -> Self {
        ReplyOn::Always
    }
}

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
    pub reply_on: ReplyOn,
}

/// The Result object returned to subcall_response. We always get the same id back
/// and then must handle success and error cases ourselves
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reply {
    pub id: u64,
    pub result: ContractResult<SubcallResponse>,
}

/// The information we get back from a successful sub-call, with full sdk events
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubcallResponse {
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
