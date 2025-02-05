use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

fn main() {
    write_api! {
        instantiate: Empty,
    }
}
