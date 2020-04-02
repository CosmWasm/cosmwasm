use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::ApiResult;
use crate::coins::Coin;
use crate::encoding::Binary;
use crate::types::HumanAddr;

pub type QueryResponse = Binary;

pub type QueryResult = ApiResult<QueryResponse>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest {
    // this queries the public API of another contract at a known address (with known ABI)
    // msg is the json-encoded QueryMsg struct
    // return value is whatever the contract returns (caller should know)
    Contract {
        contract_addr: HumanAddr,
        msg: Binary, // we pass this in as Vec<u8> to the contract, so allow any binary encoding (later, limit to rawjson?)
    },
    // This calls into the native bank module for one denomination
    // Return value is BalanceResponse
    Balance {
        address: HumanAddr,
        denom: String,
    },
    // This calls into the native bank module for all denominations.
    // Note that this may be much more expensive than Balance and should be avoided if possible.
    // Return value is AllBalanceResponse.
    AllBalances {
        address: HumanAddr,
    },
    #[cfg(feature = "staking")]
    Staking(StakingRequest),
    #[cfg(feature = "swap")]
    Swap(SwapRequest),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    // Always returns a Coin with the requested denom.
    // This may be of 0 amount if no such funds.
    pub amount: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllBalanceResponse {
    // Returns all non-zero coins held by this account.
    pub amount: Vec<Coin>,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingRequest {
    Validators {},
    // Delegations will return all delegations by the delegator,
    // or just those to the given validator (if set)
    Delegations {
        delegator: HumanAddr,
        validator: Option<HumanAddr>,
    },
}

#[cfg(feature = "swap")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapRequest {
    ExchangeRate { offer: String, ask: String },
    // Delegations will return all delegations by the delegator,
    // or just those to the given validator (if set)
    Simulate { offer: Coin, ask: String },
}

#[cfg(feature = "swap")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// ExchangeRateResponse is data format returned from SwapRequest::ExchangeRate query
pub struct ExchangeRateResponse {
    // TODO: make more complex decimal, with non-fixed digits?
    // Note: Coin only have integer representation (serialized as Strings)
    // rate is denominated in 10^-9
    // 1_000_000_000 means 1 ask for 1 offer
    // 10_000_000_000 means 10 ask for 1 offer
    // 1_000_000 means 1 ask for 1000 offer
    pub rate: u64,
}

#[cfg(feature = "swap")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// SimulateSwapResponse is data format returned from SwapRequest::Simulate query
pub struct SimulateSwapResponse {
    pub receive: Coin,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// ValidatorsResponse is data format returned from StakingRequest::Validators query
pub struct ValidatorsResponse {
    pub validators: Option<Vec<Validator>>,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    pub address: HumanAddr,
    // TODO: what other info do we want to expose?
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// DelegationsResponse is data format returned from StakingRequest::Delegations query
pub struct DelegationsResponse {
    pub delegations: Option<Vec<Delegation>>,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator: HumanAddr,
    pub validator: HumanAddr,
    pub amount: Coin,
    pub can_redelegate: bool,
    // TODO: do we want to expose more info?
}
