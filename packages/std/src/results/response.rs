use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, ReplyOn};

use super::{Attribute, CosmosMsg, Empty};
use crate::results::SubMsg;

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
///         submessages: vec![],
///         messages: vec![],
///         attributes: vec![attr("action", "instantiate")],
///         data: None,
///     })
/// }
/// ```
///
/// Mutating:
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, MessageInfo};
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
    /// Optional list of "subcalls" to make. These will be executed in order
    /// (and this contract's subcall_response entry point invoked)
    /// *before* any of the "fire and forget" messages get executed.
    pub submessages: Vec<SubMsg<T>>,
    /// After any submessages are processed, these are all dispatched in the host blockchain.
    /// If they all succeed, then the transaction is committed. If any fail, then the transaction
    /// and any local contract state changes are reverted.
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<T> Default for Response<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        Response {
            submessages: vec![],
            messages: vec![],
            attributes: vec![],
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

    pub fn add_attribute<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
    }

    pub fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(msg.into());
    }

    pub fn add_submessage<U: Into<CosmosMsg<T>>>(
        &mut self,
        id: u64,
        msg: U,
        gas_limit: Option<u64>,
        reply_on: ReplyOn,
    ) {
        let sub = SubMsg {
            id,
            msg: msg.into(),
            gas_limit,
            reply_on,
        };
        self.submessages.push(sub);
    }

    pub fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

#[cfg(test)]
mod tests {
    use super::super::BankMsg;
    use super::*;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = Response {
            submessages: vec![SubMsg {
                id: 12,
                msg: BankMsg::Send {
                    to_address: String::from("checker"),
                    amount: coins(888, "moon"),
                }
                .into(),
                gas_limit: Some(12345u64),
                reply_on: ReplyOn::Always,
            }],
            messages: vec![BankMsg::Send {
                to_address: String::from("you"),
                amount: coins(1015, "earth"),
            }
            .into()],
            attributes: vec![Attribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
            data: Some(Binary::from([0xAA, 0xBB])),
        };
        let serialized = to_vec(&original).expect("encode contract result");
        let deserialized: Response = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }
}
