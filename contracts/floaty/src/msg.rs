use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::instructions::Value;

#[cw_serde]
pub enum ValueType {
    Float,
    Int,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns valid random arguments for the given instruction
    #[returns(Vec<Value>)]
    RandomArgsFor { instruction: String, seed: u64 },
    /// Returns a list of all instructions
    #[returns(Vec<String>)]
    Instructions {},
    /// Runs the given instruction with the given arguments and returns the result
    #[returns(Value)]
    Run {
        instruction: String,
        args: Vec<Value>,
    },
}
