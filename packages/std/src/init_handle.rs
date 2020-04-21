//! Types and helpers for init and handle

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::ApiResult;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::types::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CosmosMsg {
    Bank(BankMsg),
    Contract(ContractMsg),
    // this is dangerous to use, as it ties you to one particular runtime format.
    // this makes the contract non-portable, and also fragile to break upon a hardfork
    // only safe way is to receive it from a user and hold it temporarily.
    Native { msg: Binary },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContractMsg {
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

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct InitResponse {
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type InitResult = ApiResult<InitResponse>;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct HandleResponse {
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type HandleResult = ApiResult<HandleResponse>;

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
            messages: vec![CosmosMsg::Bank(BankMsg::Send {
                from_address: HumanAddr("me".to_string()),
                to_address: HumanAddr("you".to_string()),
                amount: coins(1015, "earth"),
            })],
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
}
