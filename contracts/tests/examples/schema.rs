use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

use tests::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: Empty,
        // query: QueryMsg,
        execute: ExecuteMsg,
    }
}
