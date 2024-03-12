use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::Binary;

use super::{CosmosMsg, Empty, Event};

/// Use this to define when the contract gets a response callback.
/// If you only need it for errors or success you can select just those in order
/// to save gas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubMsg<T = Empty> {
    /// An arbitrary ID chosen by the contract.
    /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
    pub id: u64,
    /// Some arbirary data that the contract can set in an application specific way.
    /// This is just passed into the `reply` entry point and is not stored to state.
    /// Any encoding can be used. If `id` is used to identify a particular action,
    /// the encoding can also be different for each of those actions since you can match `id`
    /// first and then start processing the `payload`.
    ///
    /// The environment restricts the length of this field in order to avoid abuse. The limit
    /// is environment specific and can change over time. The initial default is 128 KiB.
    ///
    /// Unset/nil/null cannot be differentiated from empty data.
    ///
    /// On chains running CosmWasm 1.x this field will be ignored.
    #[serde(default)]
    pub payload: Binary,
    pub msg: CosmosMsg<T>,
    /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    ///
    /// Setting this to `None` means unlimited. Then the submessage execution can consume all gas of the
    /// current execution context.
    pub gas_limit: Option<u64>,
    pub reply_on: ReplyOn,
}

/// This is used for cases when we use ReplyOn::Never and the id doesn't matter
pub const UNUSED_MSG_ID: u64 = 0;

impl<T> SubMsg<T> {
    /// Creates a "fire and forget" message with the pre-0.14 semantics.
    /// Since this is just an alias for [`SubMsg::reply_never`] it is somewhat recommended
    /// to use the latter in order to make the behaviour more explicit in the caller code.
    /// But that's up to you for now.
    ///
    /// By default, the submessage's gas limit will be unlimited. Use [`SubMsg::with_gas_limit`] to change it.
    /// Setting `payload` is not advised as this will never be used.
    pub fn new(msg: impl Into<CosmosMsg<T>>) -> Self {
        Self::reply_never(msg)
    }

    /// Creates a `SubMsg` that will provide a `reply` with the given `id` if the message returns `Ok`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_on_success(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Success)
    }

    /// Creates a `SubMsg` that will provide a `reply` with the given `id` if the message returns `Err`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_on_error(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Error)
    }

    /// Create a `SubMsg` that will always provide a `reply` with the given `id`.
    ///
    /// By default, the submessage's `payload` will be empty and the gas limit will be unlimited. Use
    /// [`SubMsg::with_payload`] and [`SubMsg::with_gas_limit`] to change those.
    pub fn reply_always(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Always)
    }

    /// Create a `SubMsg` that will never `reply`. This is equivalent to standard message semantics.
    ///
    /// By default, the submessage's gas limit will be unlimited. Use [`SubMsg::with_gas_limit`] to change it.
    /// Setting `payload` is not advised as this will never be used.
    pub fn reply_never(msg: impl Into<CosmosMsg<T>>) -> Self {
        Self::reply_on(msg.into(), UNUSED_MSG_ID, ReplyOn::Never)
    }

    /// Add a gas limit to the submessage.
    /// This gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
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

    /// Add a payload to the submessage.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coins, BankMsg, Binary, ReplyOn, SubMsg};
    /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
    /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234)
    ///     .with_payload(vec![1, 2, 3, 4]);
    /// assert_eq!(sub_msg.id, 1234);
    /// assert_eq!(sub_msg.payload, Binary::new(vec![1, 2, 3, 4]));
    /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
    /// ```
    pub fn with_payload(mut self, payload: impl Into<Binary>) -> Self {
        self.payload = payload.into();
        self
    }

    fn reply_on(msg: CosmosMsg<T>, id: u64, reply_on: ReplyOn) -> Self {
        SubMsg {
            id,
            payload: Default::default(),
            msg,
            reply_on,
            gas_limit: None,
        }
    }
}

/// The result object returned to `reply`. We always get the ID from the submessage
/// back and then must handle success and error cases ourselves.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Reply {
    /// The ID that the contract set when emitting the `SubMsg`.
    /// Use this to identify which submessage triggered the `reply`.
    pub id: u64,
    /// Some arbirary data that the contract set when emitting the `SubMsg`.
    /// This is just passed into the `reply` entry point and is not stored to state.
    ///
    /// Unset/nil/null cannot be differentiated from empty data.
    ///
    /// On chains running CosmWasm 1.x this field is never filled.
    #[serde(default)]
    pub payload: Binary,
    /// The amount of gas used by the submessage,
    /// measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_used: u64,
    pub result: SubMsgResult,
}

/// This is the result type that is returned from a sub message execution.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the ComsWasm VM needs to create the same JSON representation.
///
/// Until version 1.0.0-beta5, `ContractResult<SubMsgResponse>` was used instead
/// of this type. Once serialized, the two types are the same. However, in the Rust type
/// system we want different types for clarity and documenation reasons.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use cosmwasm_std::{to_json_string, Binary, Event, SubMsgResponse, SubMsgResult};
/// #[allow(deprecated)]
/// let response = SubMsgResponse {
///     data: Some(Binary::from_base64("MTIzCg==").unwrap()),
///     events: vec![Event::new("wasm").add_attribute("fo", "ba")],
///     msg_responses: vec![],
/// };
/// let result: SubMsgResult = SubMsgResult::Ok(response);
/// assert_eq!(
///     to_json_string(&result).unwrap(),
///     r#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg==","msg_responses":[]}}"#,
/// );
/// ```
///
/// Failure:
///
/// ```
/// # use cosmwasm_std::{to_json_string, SubMsgResult, Response};
/// let error_msg = String::from("Something went wrong");
/// let result = SubMsgResult::Err(error_msg);
/// assert_eq!(to_json_string(&result).unwrap(), r#"{"error":"Something went wrong"}"#);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubMsgResult {
    Ok(SubMsgResponse),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl SubMsgResult {
    /// Converts a `SubMsgResult<S>` to a `Result<S, String>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<SubMsgResponse, String> {
        Result::<SubMsgResponse, String>::from(self)
    }

    pub fn unwrap(self) -> SubMsgResponse {
        self.into_result().unwrap()
    }

    pub fn unwrap_err(self) -> String {
        self.into_result().unwrap_err()
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, SubMsgResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, SubMsgResult::Err(_))
    }
}

impl<E: ToString> From<Result<SubMsgResponse, E>> for SubMsgResult {
    fn from(original: Result<SubMsgResponse, E>) -> SubMsgResult {
        match original {
            Ok(value) => SubMsgResult::Ok(value),
            Err(err) => SubMsgResult::Err(err.to_string()),
        }
    }
}

impl From<SubMsgResult> for Result<SubMsgResponse, String> {
    fn from(original: SubMsgResult) -> Result<SubMsgResponse, String> {
        match original {
            SubMsgResult::Ok(value) => Ok(value),
            SubMsgResult::Err(err) => Err(err),
        }
    }
}

/// The information we get back from a successful sub message execution,
/// with full Cosmos SDK events.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubMsgResponse {
    pub events: Vec<Event>,
    #[deprecated = "Deprecated in the Cosmos SDK in favor of msg_responses. If your chain is running on CosmWasm 2.0 or higher, msg_responses will be filled. For older versions, the data field is still needed since msg_responses is empty in those cases."]
    pub data: Option<Binary>,
    /// The responses from the messages emitted by the submessage.
    /// In most cases, this is equivalent to the Cosmos SDK's [MsgResponses], which usually contains a [single message].
    /// However, wasmd allows chains to translate a single contract message into multiple SDK messages.
    /// In that case all the MsgResponses from each are concatenated into this flattened `Vec`.
    ///
    /// [MsgResponses]: https://github.com/cosmos/cosmos-sdk/blob/316750cc8cd8b3296fa233f4da2e39cbcdc34517/proto/cosmos/base/abci/v1beta1/abci.proto#L106-L109
    /// [single message]: https://github.com/cosmos/cosmos-sdk/blob/v0.50.4/baseapp/baseapp.go#L1020-L1023
    #[serde(default)]
    pub msg_responses: Vec<MsgResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MsgResponse {
    pub type_url: String,
    pub value: Binary,
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::{coins, from_json, to_json_vec, Attribute, BankMsg, StdError, StdResult};

    #[test]
    fn sub_msg_new_works() {
        let msg = BankMsg::Send {
            to_address: String::from("you"),
            amount: coins(1015, "earth"),
        };
        let sub_msg: SubMsg = SubMsg::new(msg.clone());
        // id and payload don't matter since there is no reply
        assert_eq!(sub_msg.reply_on, ReplyOn::Never);
        assert_eq!(sub_msg.gas_limit, None);
        assert_eq!(sub_msg.msg, CosmosMsg::from(msg));
    }

    #[test]
    fn sub_msg_reply_never_works() {
        let msg = BankMsg::Send {
            to_address: String::from("you"),
            amount: coins(1015, "earth"),
        };
        let sub_msg: SubMsg = SubMsg::reply_never(msg.clone());
        // id and payload don't matter since there is no reply
        assert_eq!(sub_msg.reply_on, ReplyOn::Never);
        assert_eq!(sub_msg.gas_limit, None);
        assert_eq!(sub_msg.msg, CosmosMsg::from(msg));
    }

    #[test]
    fn sub_msg_reply_always_works() {
        let msg = BankMsg::Send {
            to_address: String::from("you"),
            amount: coins(1015, "earth"),
        };
        let sub_msg: SubMsg = SubMsg::reply_always(msg.clone(), 54);
        assert_eq!(sub_msg.id, 54);
        assert_eq!(sub_msg.payload, Binary::default());
        assert_eq!(sub_msg.reply_on, ReplyOn::Always);
        assert_eq!(sub_msg.gas_limit, None);
        assert_eq!(sub_msg.msg, CosmosMsg::from(msg));
    }

    #[test]
    fn sub_msg_with_gas_limit_works() {
        let msg = BankMsg::Send {
            to_address: String::from("you"),
            amount: coins(1015, "earth"),
        };
        let sub_msg: SubMsg = SubMsg::reply_never(msg);
        assert_eq!(sub_msg.gas_limit, None);
        let sub_msg = sub_msg.with_gas_limit(20);
        assert_eq!(sub_msg.gas_limit, Some(20));
    }

    #[test]
    fn sub_msg_with_payload_works() {
        let msg = BankMsg::Send {
            to_address: String::from("you"),
            amount: coins(1015, "earth"),
        };
        let sub_msg: SubMsg = SubMsg::reply_never(msg);
        assert_eq!(sub_msg.payload, Binary::default());
        let sub_msg = sub_msg.with_payload(vec![0xAA, 3, 5, 1, 2]);
        assert_eq!(sub_msg.payload, Binary::new(vec![0xAA, 3, 5, 1, 2]));
    }

    #[test]
    fn sub_msg_result_serialization_works() {
        let result = SubMsgResult::Ok(SubMsgResponse {
            data: None,
            msg_responses: vec![],
            events: vec![],
        });
        assert_eq!(
            &to_json_vec(&result).unwrap(),
            br#"{"ok":{"events":[],"data":null,"msg_responses":[]}}"#
        );

        let result = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            msg_responses: vec![MsgResponse {
                type_url: "URL".to_string(),
                value: Binary::from_base64("MTIzCg==").unwrap(),
            }],
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        });
        println!("{}", &crate::to_json_string(&result).unwrap());
        assert_eq!(
            &to_json_vec(&result).unwrap(),
            br#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg==","msg_responses":[{"type_url":"URL","value":"MTIzCg=="}]}}"#
        );

        let result: SubMsgResult = SubMsgResult::Err("broken".to_string());
        assert_eq!(&to_json_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn sub_msg_result_deserialization_works() {
        // should work without `msg_responses`
        let result: SubMsgResult = from_json(br#"{"ok":{"events":[]}}"#).unwrap();
        assert_eq!(
            result,
            SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
                msg_responses: vec![]
            })
        );

        // should work with `data` and no `msg_responses`
        // this is the case for pre-2.0 CosmWasm chains
        let result: SubMsgResult = from_json(br#"{"ok":{"events":[],"data":"aGk="}}"#).unwrap();
        assert_eq!(
            result,
            SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some(Binary::from_base64("aGk=").unwrap()),
                msg_responses: vec![]
            })
        );

        let result: SubMsgResult = from_json(
            br#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg==",
            "msg_responses":[{"type_url":"URL","value":"MTIzCg=="}]}}"#).unwrap();
        assert_eq!(
            result,
            SubMsgResult::Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                msg_responses: vec![MsgResponse {
                    type_url: "URL".to_string(),
                    value: Binary::from_base64("MTIzCg==").unwrap(),
                }],
                events: vec![Event::new("wasm").add_attribute("fo", "ba")],
            })
        );

        let result: SubMsgResult = from_json(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, SubMsgResult::Err("broken".to_string()));

        // fails for additional attributes
        let parse: StdResult<SubMsgResult> = from_json(br#"{"unrelated":321,"error":"broken"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {err:?}"),
        }
        let parse: StdResult<SubMsgResult> = from_json(br#"{"error":"broken","unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn sub_msg_result_unwrap_works() {
        let response = SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            msg_responses: vec![MsgResponse {
                type_url: "URL".to_string(),
                value: Binary::from_base64("MTIzCg==").unwrap(),
            }],
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        };
        let success = SubMsgResult::Ok(response.clone());
        assert_eq!(success.unwrap(), response);
    }

    #[test]
    #[should_panic]
    fn sub_msg_result_unwrap_panicks_for_err() {
        let failure = SubMsgResult::Err("broken".to_string());
        let _ = failure.unwrap();
    }

    #[test]
    fn sub_msg_result_unwrap_err_works() {
        let failure = SubMsgResult::Err("broken".to_string());
        assert_eq!(failure.unwrap_err(), "broken");
    }

    #[test]
    #[should_panic]
    fn sub_msg_result_unwrap_err_panics_for_ok() {
        let response = SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
            msg_responses: vec![],
        };
        let success = SubMsgResult::Ok(response);
        let _ = success.unwrap_err();
    }

    #[test]
    fn sub_msg_result_is_ok_works() {
        let success = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
            msg_responses: vec![],
        });
        let failure = SubMsgResult::Err("broken".to_string());
        assert!(success.is_ok());
        assert!(!failure.is_ok());
    }

    #[test]
    fn sub_msg_result_is_err_works() {
        let success = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
            msg_responses: vec![],
        });
        let failure = SubMsgResult::Err("broken".to_string());
        assert!(failure.is_err());
        assert!(!success.is_err());
    }

    #[test]
    fn sub_msg_result_can_convert_from_core_result() {
        let original: Result<SubMsgResponse, StdError> = Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![],
            msg_responses: vec![],
        });
        let converted: SubMsgResult = original.into();
        assert_eq!(
            converted,
            SubMsgResult::Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                events: vec![],
                msg_responses: vec![],
            })
        );

        let original: Result<SubMsgResponse, StdError> = Err(StdError::generic_err("broken"));
        let converted: SubMsgResult = original.into();
        assert_eq!(
            converted,
            SubMsgResult::Err("Generic error: broken".to_string())
        );
    }

    #[test]
    fn sub_msg_result_can_convert_to_core_result() {
        let original = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![],
            msg_responses: vec![],
        });
        let converted: Result<SubMsgResponse, String> = original.into();
        assert_eq!(
            converted,
            Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                events: vec![],
                msg_responses: vec![],
            })
        );

        let original = SubMsgResult::Err("went wrong".to_string());
        let converted: Result<SubMsgResponse, String> = original.into();
        assert_eq!(converted, Err("went wrong".to_string()));
    }

    #[test]
    fn reply_deserialization_works() {
        // 1.x reply without payload (from https://github.com/CosmWasm/cosmwasm/issues/1909)
        let reply: Reply = from_json(r#"{"gas_used":4312324,"id":75,"result":{"ok":{"events":[{"type":"hi","attributes":[{"key":"si","value":"claro"}]}],"data":"PwCqXKs="}}}"#).unwrap();
        assert_eq!(
            reply,
            Reply {
                id: 75,
                payload: Binary::default(),
                gas_used: 4312324,
                result: SubMsgResult::Ok(SubMsgResponse {
                    data: Some(Binary::from_base64("PwCqXKs=").unwrap()),
                    events: vec![Event {
                        ty: "hi".to_string(),
                        attributes: vec![Attribute {
                            key: "si".to_string(),
                            value: "claro".to_string(),
                        }]
                    }],
                    msg_responses: vec![],
                })
            }
        );

        // with payload (manually added to the above test)
        let reply: Reply = from_json(r#"{"gas_used":4312324,"id":75,"payload":"3NxjC5U=","result":{"ok":{"events":[{"type":"hi","attributes":[{"key":"si","value":"claro"}]}],"data":"PwCqXKs="}}}"#).unwrap();
        assert_eq!(
            reply,
            Reply {
                id: 75,
                payload: Binary::from_base64("3NxjC5U=").unwrap(),
                gas_used: 4312324,
                result: SubMsgResult::Ok(SubMsgResponse {
                    data: Some(Binary::from_base64("PwCqXKs=").unwrap()),
                    events: vec![Event {
                        ty: "hi".to_string(),
                        attributes: vec![Attribute {
                            key: "si".to_string(),
                            value: "claro".to_string(),
                        }]
                    }],
                    msg_responses: vec![],
                })
            }
        );
    }
}
