use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::State)]
    QueryState {},
}

#[cw_serde]
pub struct IbcPayload {
    pub response_without_ack: bool,
    pub send_async_ack_for_prev_msg: bool,
}
