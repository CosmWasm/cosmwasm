use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

use cyberpunk::msg::{ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
