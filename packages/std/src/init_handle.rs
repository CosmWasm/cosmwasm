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
        send: Vec<Coin>,
    },
    /// this instantiates a new contracts from previously uploaded wasm code
    Instantiate {
        code_id: u64,
        /// msg is the json-encoded InitMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
        /// optional human-readbale label for the contract
        label: Option<String>,
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

/// A shorthand to produce a log attribute
pub fn log<K: ToString, V: ToString>(key: K, value: V) -> LogAttribute {
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

// some proposed helpers for InitResponse, designed for chaining
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> InitResponse<T> {
    pub fn with_message(mut self, msg: CosmosMsg<T>) -> Self {
        self.messages.push(msg);
        self
    }

    pub fn with_log<U: ToString>(mut self, key: &str, value: U) -> Self {
        self.log.push(log(key, value));
        self
    }

    pub fn with_data<U: Into<Binary>>(mut self, data: U) -> Self {
        self.data = Some(data.into());
        self
    }
}

// other proposed helpers for InitResponse, mutating state
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> InitResponse<T> {
    pub fn add_message(&mut self, msg: CosmosMsg<T>) {
        self.messages.push(msg);
    }

    pub fn add_log<U: ToString>(&mut self, key: &str, value: U) {
        self.log.push(log(key, value));
    }

    pub fn add_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
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
    use crate::{coins, from_slice, to_vec, Uint128};

    #[test]
    fn init_response_chaining_helpers() {
        let msg = CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from("sender"),
            to_address: HumanAddr::from("recipient"),
            amount: coins(100, "test"),
        });

        let expected = InitResponse::<Never> {
            messages: vec![msg.clone()],
            log: vec![log("action", "demo"), log("sender", "foobar")],
            data: Some(Binary(b"sample".to_vec())),
        };

        // this is more verbose, but may be a more readable alternative to the above constructor
        let res: InitResponse<Never> = InitResponse::default()
            .with_log("action", "demo")
            .with_log("sender", &HumanAddr::from("foobar"))
            .with_message(msg.clone())
            .with_data(b"sample".to_vec());

        assert_eq!(res, expected);

        // especially when we have less data
        let expected = InitResponse::<Never> {
            messages: vec![],
            log: vec![log("init", "success")],
            data: None,
        };

        let res: InitResponse<Never> = InitResponse::default().with_log("init", "success");

        assert_eq!(res, expected);
    }

    #[test]
    fn init_response_mutating_helpers() {
        let msg = CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from("sender"),
            to_address: HumanAddr::from("recipient"),
            amount: coins(100, "test"),
        });

        let expected = InitResponse::<Never> {
            messages: vec![msg.clone()],
            log: vec![log("action", "demo"), log("sender", "foobar")],
            data: Some(Binary(b"sample".to_vec())),
        };

        let mut res: InitResponse<Never> = InitResponse::default();
        // these can be separated over various parts of the init functon
        res.add_log("action", "demo");
        res.add_log("sender", &HumanAddr::from("foobar"));
        // setting the message at the end
        res.add_message(msg.clone());
        // and the data once we get the id
        res.add_data(b"sample".to_vec());

        assert_eq!(res, expected);
    }

    #[test]
    fn log_works_for_different_types() {
        let expeceted = LogAttribute {
            key: "foo".to_string(),
            value: "42".to_string(),
        };

        assert_eq!(log("foo", "42"), expeceted);
        assert_eq!(log("foo".to_string(), "42"), expeceted);
        assert_eq!(log("foo", "42".to_string()), expeceted);
        assert_eq!(log("foo", HumanAddr::from("42")), expeceted);
        assert_eq!(log("foo", Uint128(42)), expeceted);
        assert_eq!(log("foo", 42), expeceted);
    }

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
