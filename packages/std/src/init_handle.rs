//! Types and helpers for init and handle

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::coins::Coin;
use crate::encoding::Binary;
use crate::errors::StdResult;
use crate::types::{HumanAddr, Never};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Never>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    #[cfg(feature = "staking")]
    Staking(StakingMsg),
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

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Delegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Undelegate {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        amount: Coin,
    },
    Withdraw {
        // delegator is automatically set to address of the calling contract
        validator: HumanAddr,
        /// this is the "withdraw address", the one that should receive the rewards
        /// if None, then use delegator address
        recipient: Option<HumanAddr>,
    },
    Redelegate {
        // delegator is automatically set to address of the calling contract
        src_validator: HumanAddr,
        dst_validator: HumanAddr,
        amount: Coin,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// this dispatches a call to another contract at a known address (with known ABI)
    Execute {
        contract_addr: HumanAddr,
        /// msg is the json-encoded HandleMsg struct (as raw Binary)
        msg: Binary,
        send: Option<Vec<Coin>>,
    },
    /// this instantiates a new contracts from previously uploaded wasm code
    Instantiate {
        code_id: u64,
        /// msg is the json-encoded InitMsg struct (as raw Binary)
        msg: Binary,
        send: Option<Vec<Coin>>,
    },
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
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
pub struct InitResponse<T = Never>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type InitResult<U = Never> = StdResult<InitResponse<U>>;

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
pub struct HandleResponse<T = Never>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>, // abci defines this as string
    pub data: Option<Binary>,   // abci defines this as bytes
}

pub type HandleResult<U = Never> = StdResult<HandleResponse<U>>;

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
    use crate::errors::StdError;
    use crate::{coins, from_slice, to_vec};

    #[test]
    fn can_deser_error_result() {
        let fail = InitResult::Err(StdError::Unauthorized { backtrace: None });
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
