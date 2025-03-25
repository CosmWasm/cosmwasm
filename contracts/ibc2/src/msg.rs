use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum PacketMsg {
    Increment {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::State)]
    QueryState {},
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {
        channel_id: String,
        destination_port: String,
    },
}
