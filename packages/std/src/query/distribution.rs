use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Addr;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionQuery {
    // https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L792-L795
    DelegatorWithdrawAddress { delegator_address: String },
}

// https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L832-L835
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct DelegatorWithdrawAddressResponse {
    pub withdraw_address: Addr,
}

impl DelegatorWithdrawAddressResponse {
    /// Constructor for testing frameworks such as cw-multi-test.
    /// This is required because query response types should be #[non_exhaustive].
    /// As a contract developer you should not need this constructor since
    /// query responses are constructed for you via deserialization.
    ///
    /// Warning: This is for cw-multi-test use only and can change at any time.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn from_parts(withdraw_address: Addr) -> Self {
        Self { withdraw_address }
    }
}
