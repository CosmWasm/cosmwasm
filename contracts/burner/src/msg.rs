use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct MigrateMsg {
    pub payout: String,
}

/// A placeholder where we don't take any input
#[cw_serde]
pub struct InstantiateMsg {}
