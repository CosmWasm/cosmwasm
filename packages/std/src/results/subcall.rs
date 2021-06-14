use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, ContractResult};

use super::{Attribute, CosmosMsg, Empty};

/// Use this to define when the contract gets a response callback.
/// If you only need it for errors or success you can select just those in order
/// to save gas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

/// A submessage that will guarantee a `reply` call on success or error, depending on
/// the `reply_on` setting. If you do not need to process the result, use regular messages instead.
///
/// Note: On error the submessage execution will revert any partial state changes due to this message,
/// but not revert any state changes in the calling contract. If this is required, it must be done
/// manually in the `reply` entry point.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// An arbitrary ID chosen by the contract.
    /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
    pub id: u64,
    pub msg: CosmosMsg<T>,
    pub gas_limit: Option<u64>,
    pub reply_on: ReplyOn,
}

/// The result object returned to `reply`. We always get the ID from the submessage
/// back and then must handle success and error cases ourselves.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reply {
    /// The ID that the contract set when emitting the `SubMsg`.
    /// Use this to identify which submessage triggered the `reply`.
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
