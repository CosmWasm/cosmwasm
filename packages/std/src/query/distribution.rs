use cosmwasm_schema::cw_serde;

use super::query_response::QueryResponseType;

#[non_exhaustive]
#[cw_serde]
#[derive(Eq)]
pub enum DistributionQuery {
    // https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L792-L795
    DelegatorWithdrawAddress { delegator_address: String },
}

// https://github.com/cosmos/cosmos-sdk/blob/4f6f6c00021f4b5ee486bbb71ae2071a8ceb47c9/x/distribution/types/query.pb.go#L832-L835
#[cw_serde]
#[derive(Eq, Default)]
#[non_exhaustive]
pub struct DelegatorWithdrawAddressResponse {
    pub withdraw_address: String,
}

impl QueryResponseType for DelegatorWithdrawAddressResponse {}
