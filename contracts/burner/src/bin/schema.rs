use cosmwasm_schema::write_api;

use burner::msg::{InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
    }
}
