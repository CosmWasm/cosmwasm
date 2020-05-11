use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::coins::Coin;
use crate::encoding::Binary;
use crate::errors::StdResult;
use crate::types::HumanAddr;

pub type QueryResponse = Binary;

pub type QueryResult = StdResult<QueryResponse>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest<T> {
    Bank(BankQuery),
    Custom(T),
    Staking(StakingQuery),
    Wasm(WasmQuery),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankQuery {
    /// This calls into the native bank module for one denomination
    /// Return value is BalanceResponse
    Balance { address: HumanAddr, denom: String },
    /// This calls into the native bank module for all denominations.
    /// Note that this may be much more expensive than Balance and should be avoided if possible.
    /// Return value is AllBalanceResponse.
    AllBalances { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmQuery {
    /// this queries the public API of another contract at a known address (with known ABI)
    /// return value is whatever the contract returns (caller should know)
    Smart {
        contract_addr: HumanAddr,
        /// msg is the json-encoded QueryMsg struct
        msg: Binary,
    },
    /// this queries the raw kv-store of the contract.
    /// returns the raw, unparsed data stored at that key (or `Ok(Err(StdError:NotFound{}))` if missing)
    Raw {
        contract_addr: HumanAddr,
        /// Key is the raw key used in the contracts Storage
        key: Binary,
    },
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<BankQuery> for QueryRequest<T> {
    fn from(msg: BankQuery) -> Self {
        QueryRequest::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<StakingQuery> for QueryRequest<T> {
    fn from(msg: StakingQuery) -> Self {
        QueryRequest::Staking(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<WasmQuery> for QueryRequest<T> {
    fn from(msg: WasmQuery) -> Self {
        QueryRequest::Wasm(msg)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    /// Always returns a Coin with the requested denom.
    /// This may be of 0 amount if no such funds.
    pub amount: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllBalanceResponse {
    /// Returns all non-zero coins held by this account.
    pub amount: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingQuery {
    /// Returns all registered Validators on the system
    Validators {},
    /// Delegations will return all delegations by the delegator,
    /// or just those to the given validator (if set)
    Delegations {
        delegator: HumanAddr,
        validator: Option<HumanAddr>,
    },
}

/// ValidatorsResponse is data format returned from StakingRequest::Validators query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    pub address: HumanAddr,
    pub commission: Billionth,
    pub max_commission: Billionth,
    /// TODO: what units are these (in terms of time)?
    pub max_change_rate: Billionth,
}

/// DelegationsResponse is data format returned from StakingRequest::Delegations query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator: HumanAddr,
    pub validator: HumanAddr,
    /// How much we have locked in the delegation
    pub amount: Coin,
    /// If true, then a Redelegate command will work now, otherwise you may have to wait more
    pub can_redelegate: bool,
    /// How much we can currently withdraw
    pub accumulated_rewards: Coin,
    // TODO: do we want to expose more info?
}

/// Billionth represents a fixed-point decimal value with 9 fractional digits.
/// That is Billionth(1_000_000_000) == 1
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, JsonSchema)]
pub struct Billionth(u64);

impl Billionth {
    pub fn one() -> Billionth {
        Billionth(1_000_000_000)
    }

    // convert integer % into Billionth units
    pub fn percent(percent: u64) -> Billionth {
        Billionth(percent * 10_000_000)
    }

    // convert permille (1/1000) into Billionth units
    pub fn permille(permille: u64) -> Billionth {
        Billionth(permille * 1_000_000)
    }
}
