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
    Bank(BankQuery),
    #[cfg(feature = "staking")]
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
    /// returns the raw, unparsed data stored at that key (or `Ok(Err(ApiError:NotFound{}))` if missing)
    Raw {
        contract_addr: HumanAddr,
        /// Key is the raw key used in the contracts Storage
        key: Binary,
    },
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

#[cfg(feature = "staking")]
pub use staking::{Delegation, DelegationsResponse, StakingQuery, Validator, ValidatorsResponse};

#[cfg(feature = "staking")]
mod staking {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::coins::Coin;
    use crate::types::HumanAddr;

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
    #[cfg(feature = "staking")]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ValidatorsResponse {
        pub validators: Vec<Validator>,
    }

    #[cfg(feature = "staking")]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Validator {
        pub address: HumanAddr,
        /// rates are denominated in 10^-6 - 1_000_000 (max) = 100%, 10_000 = 1%
        /// TODO: capture this in some Dec type?
        pub commission: u64,
        pub max_commission: u64,
        /// TODO: what units are these (in terms of time)?
        pub max_change_rate: u64,
    }

    /// DelegationsResponse is data format returned from StakingRequest::Delegations query
    #[cfg(feature = "staking")]
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct DelegationsResponse {
        pub delegations: Vec<Delegation>,
    }

    #[cfg(feature = "staking")]
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
}
