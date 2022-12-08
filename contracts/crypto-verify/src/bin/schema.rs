use cosmwasm_schema::write_api;

use crypto_verify::msg::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
    }
}
