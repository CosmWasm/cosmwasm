use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use ibc2::contract::QueryMsg;

fn main() {
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
    }
}
