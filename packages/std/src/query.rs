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
    // ExchangeRate will return the rate between just this pair.
    ExchangeRate { offer: String, ask: String },
    // ExchangeRates will return the exchange rate between offer denom and all supported asks
    ExchangeRates { offer: String },
    // Delegations will return all delegations by the delegator,
    // or just those to the given validator (if set)
    Simulate { offer: Coin, ask: String },
}

#[cfg(feature = "swap")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// ExchangeRateResponse is data format returned from SwapRequest::ExchangeRate query
pub struct ExchangeRateResponse {
    pub ask: String,
    // rate is denominated in 10^-6
    // 1_000_000 means 1 ask for 1 offer
    // 10_000_000 means 10 ask for 1 offer
    // 1_000 means 1 ask for 1000 offer
    pub rate: u64,
}

#[cfg(feature = "swap")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// ExchangeRatesResponse is data format returned from SwapRequest::ExchangeRates query
pub struct ExchangeRatesResponse {
    pub rates: Vec<ExchangeRateResponse>,
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
    pub validators: Vec<Validator>,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    pub address: HumanAddr,
    // rates are denominated in 10^-6 - 1_000_000 (max) = 100%, 10_000 = 1%
    // TODO: capture this in some Dec type?
    pub commission: u64,
    pub max_commission: u64,
    // what units are these (in terms of time)?
    pub max_change_rate: u64,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// DelegationsResponse is data format returned from StakingRequest::Delegations query
pub struct DelegationsResponse {
    pub delegations: Vec<Delegation>,
}

#[cfg(feature = "staking")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator: HumanAddr,
    pub validator: HumanAddr,
    pub amount: Coin,
    pub can_redelegate: bool,
    // Review this: this is how much we can withdraw
    pub accumulated_rewards: Coin,
    // TODO: do we want to expose more info?
}
