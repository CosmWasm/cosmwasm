use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns a list of all instructions
    #[returns(Vec<String>)]
    Instructions {},
    /// Performs a huge amount of floating point operations and hashes them together
    #[returns(u64)]
    Run { instruction: String, seed: u64 },
}
