//! Types and helpers for init and handle

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::api::ApiResult;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::types::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = NoMsg>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    Wasm(WasmMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    // this moves tokens in the underlying sdk
    Send {
        from_address: HumanAddr,
        to_address: HumanAddr,
        amount: Vec<Coin>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// NoMsg can never be instantiated and is a no-op placeholder for
/// those contracts that don't explicitly set a custom message.
pub enum NoMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    // this dispatches a call to another contract at a known address (with known ABI)
    // msg is the json-encoded HandleMsg struct
    Execute {
        contract_addr: HumanAddr,
        msg: Binary, // we pass this in as Vec<u8> to the contract, so allow any binary encoding (later, limit to rawjson?)
        send: Option<Vec<Coin>>,
    },
    Instantiate {
        code_id: u64,
        msg: Binary, // we pass this in as Vec<u8> to the contract, so allow any binary encoding (later, limit to rawjson?)
        send: Option<Vec<Coin>>,
    },
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct LogAttribute {
    pub key: String,
    pub value: String,
}

/// A shorthand to produce log messages
pub fn log(key: &str, value: &str) -> LogAttribute {
    LogAttribute {
        key: key.to_string(),
        value: value.to_string(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitResponse<T = NoMsg>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type InitResult<U = NoMsg> = ApiResult<InitResponse<U>>;

impl<T> Default for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        InitResponse {
            messages: vec![],
            log: vec![],
            data: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleResponse<T = NoMsg>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type HandleResult<U = NoMsg> = ApiResult<HandleResponse<U>>;

impl<T> Default for HandleResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::ApiError;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_deser_error_result() {
        let fail = InitResult::Err(ApiError::Unauthorized {});
        let bin = to_vec(&fail).expect("encode contract result");
        println!("error: {}", std::str::from_utf8(&bin).unwrap());
        let back: InitResult = from_slice(&bin).expect("decode contract result");
        assert_eq!(fail, back);
    }

    #[test]
    fn can_deser_ok_result() {
        let send = InitResult::Ok(InitResponse {
            messages: vec![BankMsg::Send {
                from_address: HumanAddr("me".to_string()),
                to_address: HumanAddr("you".to_string()),
                amount: coins(1015, "earth"),
            }
            .into()],
            log: vec![LogAttribute {
                key: "action".to_string(),
                value: "release".to_string(),
            }],
            data: None,
        });
        let bin = to_vec(&send).expect("encode contract result");
        println!("ok: {}", std::str::from_utf8(&bin).unwrap());
        let back: InitResult = from_slice(&bin).expect("decode contract result");
        assert_eq!(send, back);
    }

    #[test]
    fn msg_from_works() {
        let from_address = HumanAddr("me".to_string());
        let to_address = HumanAddr("you".to_string());
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send {
            from_address,
            to_address,
            amount,
        };
        let msg: CosmosMsg = bank.clone().into();
        match msg {
            CosmosMsg::Bank(msg) => assert_eq!(bank, msg),
            _ => panic!("must encode in Bank variant"),
        }
    }
}
