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
    /// Never make a callback - this is like the original CosmosMsg semantics
    Never,
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

/// This is used for cases when we use ReplyOn::Never and the id doesn't matter
pub const UNUSED_MSG_ID: u64 = 123456789;

/// We implement thisas a shortcut so all existing code doesn't break.
/// Up to 0.14, we could do something like:
///   let messages = vec![BankMsg::Send { .. }.into()];
/// In order to construct the response.
///
/// With 0.15, we move to requiring SubMsg there, but this allows the same
/// `.into()` call to convert the BankMsg into a proper SubMsg with no reply.
impl<M, T> From<M> for SubMsg<T>
where
    M: Into<CosmosMsg<T>>,
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    #[inline]
    fn from(msg: M) -> SubMsg<T> {
        call(msg)
    }
}

/// call takes eg. BankMsg::Send{} and wraps it into a SubMsg with normal message sematics (no reply)
pub fn call<M, T>(msg: M) -> SubMsg<T>
where
    M: Into<CosmosMsg<T>>,
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    SubMsg {
        id: UNUSED_MSG_ID,
        msg: msg.into(),
        reply_on: ReplyOn::Never,
        gas_limit: None,
    }
}

impl<T> SubMsg<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// new takes eg. BankMsg::Send{} and sets up for a reply. No gas limit is set.
    pub fn new<M: Into<CosmosMsg<T>>>(msg: M, id: u64, reply_on: ReplyOn) -> Self {
        SubMsg {
            id,
            msg: msg.into(),
            reply_on,
            gas_limit: None,
        }
    }

    /// new_with_limit is like new but allows setting a gas limit
    pub fn new_with_limit<M: Into<CosmosMsg<T>>>(
        msg: M,
        id: u64,
        reply_on: ReplyOn,
        gas_limit: u64,
    ) -> Self {
        SubMsg {
            id,
            msg: msg.into(),
            reply_on,
            gas_limit: Some(gas_limit),
        }
    }
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

/// A full Cosmos SDK event as documented in
/// https://docs.cosmos.network/v0.42/core/events.html.
///
/// This version uses string attributes (similar to
/// https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/base/abci/v1beta1/abci.proto#L56-L70),
/// which then get magically converted to bytes for Tendermint somewhere between
/// the Rust-Go interface, JSON deserialization and the `NewEvent` call in Cosmos SDK.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    /// The event type. This is renamed to "kind" because "type" is reserved in Rust. This sucks, we know.
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
