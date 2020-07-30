//! Types and helpers for init and handle

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

use crate::addresses::HumanAddr;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::errors::{StdError, StdResult};
use crate::types::Empty;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Empty>
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
pub struct InitResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>,
}

pub type InitResult<U = Empty> = StdResult<InitResponse<U>>;

impl<T> Default for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        InitResponse {
            messages: vec![],
            log: vec![],
        }
    }
}

impl<T> TryFrom<Context<T>> for InitResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    type Error = StdError;

    fn try_from(ctx: Context<T>) -> Result<Self, Self::Error> {
        if ctx.data.is_some() {
            Err(StdError::generic_err(
                "cannot convert Context with data to InitResponse",
            ))
        } else {
            Ok(InitResponse {
                messages: ctx.messages,
                log: ctx.log,
            })
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HandleResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>,
    pub data: Option<Binary>,
}

pub type HandleResult<U = Empty> = StdResult<HandleResponse<U>>;

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

impl<T> From<Context<T>> for HandleResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        HandleResponse {
            messages: ctx.messages,
            log: ctx.log,
            data: ctx.data,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateResponse<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub messages: Vec<CosmosMsg<T>>,
    pub log: Vec<LogAttribute>,
    pub data: Option<Binary>,
}

pub type MigrateResult<U = Empty> = StdResult<MigrateResponse<U>>;

impl<T> Default for MigrateResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        MigrateResponse {
            messages: vec![],
            log: vec![],
            data: None,
        }
    }
}

impl<T> From<Context<T>> for MigrateResponse<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn from(ctx: Context<T>) -> Self {
        MigrateResponse {
            messages: ctx.messages,
            log: ctx.log,
            data: ctx.data,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Context<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    messages: Vec<CosmosMsg<T>>,
    log: Vec<LogAttribute>,
    data: Option<Binary>,
}

impl<T> Default for Context<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    fn default() -> Self {
        Context {
            messages: vec![],
            log: vec![],
            data: None,
        }
    }
}

impl<T> Context<T>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    pub fn new() -> Self {
        Context::default()
    }

    pub fn add_log<K: ToString, V: ToString>(&mut self, key: K, value: V) {
        self.log.push(log(key, value));
    }

    pub fn add_message<U: Into<CosmosMsg<T>>>(&mut self, msg: U) {
        self.messages.push(msg.into());
    }

    pub fn set_data<U: Into<Binary>>(&mut self, data: U) {
        self.data = Some(data.into());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::StdError;
    use crate::{coins, from_slice, to_vec, Uint128};
    use std::convert::TryInto;

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

    #[test]
    fn empty_context() {
        let ctx = Context::new();

        let init: InitResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, InitResponse::default());

        let init: HandleResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, HandleResponse::default());

        let init: MigrateResponse = ctx.clone().try_into().unwrap();
        assert_eq!(init, MigrateResponse::default());
    }

    #[test]
    fn full_context() {
        let mut ctx = Context::new();

        // build it up with the builder commands
        ctx.add_log("sender", &HumanAddr::from("john"));
        ctx.add_log("action", "test");
        ctx.add_message(BankMsg::Send {
            from_address: HumanAddr::from("goo"),
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        });

        // and this is what is should return
        let expected_log = vec![log("sender", "john"), log("action", "test")];
        let expected_msgs = vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr::from("goo"),
            to_address: HumanAddr::from("foo"),
            amount: coins(128, "uint"),
        })];
        let expected_data = Some(Binary::from(b"banana"));

        // try InitResponse before setting data
        let init: InitResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&init.messages, &expected_msgs);
        assert_eq!(&init.log, &expected_log);

        ctx.set_data(b"banana");
        // should fail with data set
        let init_err: StdResult<InitResponse> = ctx.clone().try_into();
        match init_err.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "cannot convert Context with data to InitResponse")
            }
            e => panic!("Unexpected error: {}", e),
        }

        // try Handle with everything set
        let handle: HandleResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&handle.messages, &expected_msgs);
        assert_eq!(&handle.log, &expected_log);
        assert_eq!(&handle.data, &expected_data);

        // try Migrate with everything set
        let migrate: MigrateResponse = ctx.clone().try_into().unwrap();
        assert_eq!(&migrate.messages, &expected_msgs);
        assert_eq!(&migrate.log, &expected_log);
        assert_eq!(&migrate.data, &expected_data);
    }
}
