use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Binary, Empty};

use super::attribute::Attribute;
use super::cosmos_msg::CosmosMsg;

/// A response of the contract entry point `init`.
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
/// # use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, MigrateResponse};
/// # type InitMsg = ();
/// #
/// use cosmwasm_std::{attr, InitResponse, StdResult};
///
/// pub fn init(
///     deps: DepsMut,
///     _env: Env,
///     _info: MessageInfo,
///     msg: InitMsg,
/// ) -> StdResult<InitResponse> {
///     // ...
///
///     Ok(InitResponse {
///         messages: vec![],
///         attributes: vec![attr("action", "init")],
///         data: None,
///     })
/// }
/// ```
///
/// Mutating:
///
/// ```
/// # use cosmwasm_std::{coins, BankMsg, Binary, DepsMut, Env, HumanAddr, MessageInfo, MigrateResponse};
/// # type InitMsg = ();
/// # type MyError = ();
/// #
/// use cosmwasm_std::InitResponse;
///
/// pub fn init(
///     deps: DepsMut,
///     _env: Env,
///     info: MessageInfo,
///     msg: InitMsg,
/// ) -> Result<InitResponse, MyError> {
///     let mut response = InitResponse::new();
///     // ...
///     response.add_attribute("Let the", "hacking begin");
///     // ...
///     response.add_message(BankMsg::Send {
///         to_address: HumanAddr::from("recipient"),
///         amount: coins(128, "uint"),
///     });
///     response.add_attribute("foo", "bar");
///     // ...
///     response.set_data(Binary::from(b"the result data"));
///     Ok(response)
/// }
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    /// The attributes that will be emitted as part of a "wasm" event
    pub attributes: Vec<Attribute>,
    pub data: Option<Binary>,
}

impl<T> Default for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        InitResponse {
            messages: vec![],
            attributes: vec![],
            data: None,
        }
    }
}

impl<T> InitResponse<T>
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

    pub fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

#[cfg(test)]
mod tests {
    use super::super::BankMsg;
    use super::*;
    use crate::addresses::HumanAddr;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_serialize_and_deserialize_init_response() {
        let original = InitResponse {
            messages: vec![BankMsg::Send {
                to_address: HumanAddr::from("you"),
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
        let deserialized: InitResponse = from_slice(&serialized).expect("decode contract result");
        assert_eq!(deserialized, original);
    }
}
