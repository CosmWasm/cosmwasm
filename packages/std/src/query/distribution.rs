use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionQuery {
    // https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L792-L795
    DelegatorWithdrawAddress { delegator_address: String },
}

// https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L832-L835
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DelegatorWithdrawAddressResponse {
    pub withdraw_address: String,
}

impl QueryResponseType for DelegatorWithdrawAddressResponse {}
