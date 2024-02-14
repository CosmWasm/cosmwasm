use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;

use crate::state::CallbackStats;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns stats about what callbacks have been received
    #[returns(CallbackStats)]
    CallbackStats {},
}

#[cw_serde]
pub struct ExecuteMsg {
    /// Address on the destination chain
    pub to_address: String,
    /// The channel to send the packet through
    pub channel_id: String,
    /// The amount of seconds from now the transfer should timeout at
    pub timeout_seconds: Uint64,
}
