use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, Event};

use super::{Attribute, CosmosMsg, Empty, SubMsg};

/// A response of a contract entry point, such as `instantiate`, `execute` or `migrate`.
///
/// This type can be constructed directly at the end of the call. Alternatively a
/// mutable response instance can be created early in the contract's logic and
/// incrementally be updated.
///
/// ## Examples
///
/// Direct:
///
/// ```
/// # use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo};
/// # type InstantiateMsg = ();
/// #
/// use cosmwasm_std::{attr, Response, StdResult};
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     _info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> StdResult<Response> {
///     // ...
///
///     Ok(Response {
///         messages: vec![],
///         attributes: vec![attr("action", "instantiate")],
///         events: vec![],
///         data: None,
///     })
/// }
/// ```
///
/// Mutating:
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, MessageInfo, SubMsg};
/// # type InstantiateMsg = ();
/// # type MyError = ();
/// #
/// use cosmwasm_std::Response;
///
/// pub fn instantiate(
///     deps: DepsMut,
///     _env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, MyError> {
///     let mut response = Response::new();
///     // ...
///     response.add_attribute("Let the", "hacking begin");
///     // ...
///     response.add_message(BankMsg::Send {
///         to_address: String::from("recipient"),
///         amount: coins(128, "uint"),
///     });
///     response.add_attribute("foo", "bar");
///     // ...
///     response.set_data(Binary::from(b"the result data"));
///     Ok(response)
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Response<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Optional list of messages to pass. These will be executed in order.
    /// If the ReplyOn variant matches the result (Always, Success on Ok, Error on Err),
    /// the runtime will invoke this contract's `reply` entry point
    /// after execution. Otherwise, they act like "fire and forget".
    /// Use `SubMsg::new` to create messages with the older "fire and forget" semantics.
    pub messages: Vec<SubMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

impl<T> Default for Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        Response {
            messages: vec![],
            attributes: vec![],
            events: vec![],
            data: None,
        }
    }
}

impl<T> Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attribute included in the main `wasm` event.
    pub fn add_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
    }

    /// This creates a "fire and forget" message, by using `SubMsg::new()` to wrap it,
    /// and adds it to the list of messages to process.
    pub fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(SubMsg::new(msg));
    }

    /// This takes an explicit SubMsg (creates via eg. `reply_on_error`)
    /// and adds it to the list of messages to process.
    pub fn add_submessage(&mut self, msg: SubMsg<T>) {
        self.messages.push(msg);
    }

    /// Adds an extra event to the response, separate from the main `wasm` event
    /// that is always created.
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

#[cfg(test)]
mod tests {
    use super::super::BankMsg;
    use super::*;
    use crate::results::subcall::{ReplyOn, UNUSED_MSG_ID};
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = Response {
            messages: vec![
                SubMsg {
                    id: 12,
                    msg: BankMsg::Send {
                        to_address: String::from("checker"),
                        amount: coins(888, "moon"),
                    }
                    .into(),
                    gas_limit: Some(12345u64),
                    reply_on: ReplyOn::Always,
                },
                SubMsg {
                    id: UNUSED_MSG_ID,
                    msg: BankMsg::Send {
                        to_address: String::from("you"),
                        amount: coins(1015, "earth"),
                    }
                    .into(),
                    gas_limit: None,
                    reply_on: ReplyOn::Never,
                },
            ],
            attributes: vec![Attribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
            events: vec![],
            data: Some(Binary::from([0xAA, 0xBB])),
        };
        let serialized = to_vec(&original).expect("encode contract result");
        let deserialized: Response = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }
}
