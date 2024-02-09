use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use ibc_callbacks::msg::{ExecuteMsg, QueryMsg};

fn main() {
    // Clear & write standard API
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
