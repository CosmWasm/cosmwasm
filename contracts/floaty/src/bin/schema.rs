use cosmwasm_schema::write_api;

use cosmwasm_std::Empty;
use floaty::msg::QueryMsg;

fn main() {
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
    }
}
