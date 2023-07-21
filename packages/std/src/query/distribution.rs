use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Addr;

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionQuery {
    // https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L792-L795
    DelegatorWithdrawAddress {
        delegator_address: String,
    },
    // https://github.com/cosmos/cosmos-sdk/blob/e3482f2d4142c55f9dc3f47a321b56610a11492c/x/distribution/types/query.pb.go#L525-L532
    #[cfg(feature = "cosmwasm_1_4")]
    DelegationRewards {
        delegator_address: String,
        validator_address: String,
    },
}

// https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L832-L835
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DelegatorWithdrawAddressResponse {
    pub withdraw_address: Addr,
}

impl_response_constructor!(DelegatorWithdrawAddressResponse, withdraw_address: Addr);

impl QueryResponseType for DelegatorWithdrawAddressResponse {}

// https://github.com/cosmos/cosmos-sdk/blob/e3482f2d4142c55f9dc3f47a321b56610a11492c/x/distribution/types/query.pb.go#L567-L572
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationRewardsResponse {
    pub rewards: Vec<DecCoin>,
}

impl_response_constructor!(DelegationRewardsResponse, rewards: Vec<DecCoin>);
impl QueryResponseType for DelegationRewardsResponse {}

/// A coin type with decimal amount.
/// Modeled after the Cosmos SDK's `DecCoin` type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DecCoin {
    pub denom: String,
    pub amount: crate::Decimal,
}
