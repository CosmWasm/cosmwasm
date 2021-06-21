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
pub const UNUSED_MSG_ID: u64 = 0;

impl<T> SubMsg<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// new creates a "fire and forget" message with the pre-0.14 semantics
    pub fn new<M: Into<CosmosMsg<T>>>(msg: M) -> Self {
        SubMsg {
            id: UNUSED_MSG_ID,
            msg: msg.into(),
            reply_on: ReplyOn::Never,
            gas_limit: None,
        }
    }

    /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Ok`
    pub fn reply_on_success<M: Into<CosmosMsg<T>>>(msg: M, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Success)
    }

    /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Err`
    pub fn reply_on_error<M: Into<CosmosMsg<T>>>(msg: M, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Error)
    }

    /// create a `SubMsg` that will always provide a `reply` with the given id
    pub fn reply_always<M: Into<CosmosMsg<T>>>(msg: M, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Always)
    }

    /// Add a gas limit to the message.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coins, BankMsg, ReplyOn, SubMsg};
    /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
    /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234).with_gas_limit(60_000);
    /// assert_eq!(sub_msg.id, 1234);
    /// assert_eq!(sub_msg.gas_limit, Some(60_000));
    /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
    /// ```
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    fn reply_on(msg: CosmosMsg<T>, id: u64, reply_on: ReplyOn) -> Self {
        SubMsg {
            id,
            msg,
            reply_on,
            gas_limit: None,
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
    /// Create a new event with the given type and an empty list of attributes.
    pub fn new(kind: impl Into<String>) -> Self {
        Event {
            kind: kind.into(),
            attributes: Vec::with_capacity(10),
        }
    }

    /// Add an attribute to the event.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr;

    #[test]
    fn event_construction() {
        let event_direct = Event {
            kind: "test".to_string(),
            attributes: vec![attr("foo", "bar"), attr("bar", "baz")],
        };
        let event_builder = Event::new("test").attr("foo", "bar").attr("bar", "baz");

        assert_eq!(event_direct, event_builder);
    }
}
