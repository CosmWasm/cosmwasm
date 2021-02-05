use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::addresses::HumanAddr;
use crate::binary::Binary;
use crate::coins::Coin;
#[cfg(feature = "stargate")]
use crate::ibc::IbcQuery;
use crate::math::Decimal;
use crate::types::Empty;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryRequest<C: CustomQuery> {
    Bank(BankQuery),
    Custom(C),
    Staking(StakingQuery),
    /// A Stargate query encoded the same way as abci_query, with path and protobuf encoded Data.
    /// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md)
    /// The response is also protobuf encoded. The caller is responsible for compiling the proper protobuf definitions
    #[cfg(feature = "stargate")]
    Stargate {
        /// this is the fully qualified service path used for routing,
        /// eg. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance
        path: String,
        /// this is the expected protobuf message type (not any), binary encoded
        data: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcQuery),
    Wasm(WasmQuery),
}

#[non_exhaustive]
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

/// A trait that is required to avoid conflicts with other query types like BankQuery and WasmQuery
/// in generic implementations.
/// You need to implement it in your custom query type.
///
/// # Examples
///
/// ```
/// # use cosmwasm_std::CustomQuery;
/// # use schemars::JsonSchema;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// #[serde(rename_all = "snake_case")]
/// pub enum MyCustomQuery {
///     Ping {},
///     Capitalized { text: String },
/// }
///
/// impl CustomQuery for MyCustomQuery {}
/// ```
pub trait CustomQuery: Serialize {}

impl CustomQuery for Empty {}

#[non_exhaustive]
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
    /// returns the raw, unparsed data stored at that key, which may be an empty vector if not present
    Raw {
        contract_addr: HumanAddr,
        /// Key is the raw key used in the contracts Storage
        key: Binary,
    },
}

impl<C: CustomQuery> From<BankQuery> for QueryRequest<C> {
    fn from(msg: BankQuery) -> Self {
        QueryRequest::Bank(msg)
    }
}

impl<C: CustomQuery> From<C> for QueryRequest<C> {
    fn from(msg: C) -> Self {
        QueryRequest::Custom(msg)
    }
}

#[cfg(feature = "staking")]
impl<C: CustomQuery> From<StakingQuery> for QueryRequest<C> {
    fn from(msg: StakingQuery) -> Self {
        QueryRequest::Staking(msg)
    }
}

impl<C: CustomQuery> From<WasmQuery> for QueryRequest<C> {
    fn from(msg: WasmQuery) -> Self {
        QueryRequest::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl<C: CustomQuery> From<IbcQuery> for QueryRequest<C> {
    fn from(msg: IbcQuery) -> Self {
        QueryRequest::Ibc(msg)
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

#[cfg(feature = "stargate")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StargateResponse {
    /// This is the protobuf response, binary encoded.
    /// The caller is responsible for knowing how to parse.
    pub response: Binary,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingQuery {
    /// Returns the denomination that can be bonded (if there are multiple native tokens on the chain)
    BondedDenom {},
    /// AllDelegations will return all delegations by the delegator
    AllDelegations { delegator: HumanAddr },
    /// Delegation will return more detailed info on a particular
    /// delegation, defined by delegator/validator pair
    Delegation {
        delegator: HumanAddr,
        validator: HumanAddr,
    },
    /// Returns all registered Validators on the system
    Validators {},
}

/// BondedDenomResponse is data format returned from StakingRequest::BondedDenom query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BondedDenomResponse {
    pub denom: String,
}

/// DelegationsResponse is data format returned from StakingRequest::AllDelegations query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllDelegationsResponse {
    pub delegations: Vec<Delegation>,
}

/// Delegation is basic (cheap to query) data about a delegation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Delegation {
    pub delegator: HumanAddr,
    pub validator: HumanAddr,
    /// How much we have locked in the delegation
    pub amount: Coin,
}

impl From<FullDelegation> for Delegation {
    fn from(full: FullDelegation) -> Self {
        Delegation {
            delegator: full.delegator,
            validator: full.validator,
            amount: full.amount,
        }
    }
}

/// DelegationResponse is data format returned from StakingRequest::Delegation query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationResponse {
    pub delegation: Option<FullDelegation>,
}

/// FullDelegation is all the info on the delegation, some (like accumulated_reward and can_redelegate)
/// is expensive to query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FullDelegation {
    pub delegator: HumanAddr,
    pub validator: HumanAddr,
    /// How much we have locked in the delegation
    pub amount: Coin,
    /// can_redelegate captures how much can be immediately redelegated.
    /// 0 is no redelegation and can_redelegate == amount is redelegate all
    /// but there are many places between the two
    pub can_redelegate: Coin,
    /// How much we can currently withdraw
    pub accumulated_rewards: Vec<Coin>,
}

/// ValidatorsResponse is data format returned from StakingRequest::Validators query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Validator {
    pub address: HumanAddr,
    pub commission: Decimal,
    pub max_commission: Decimal,
    /// TODO: what units are these (in terms of time)?
    pub max_change_rate: Decimal,
}
