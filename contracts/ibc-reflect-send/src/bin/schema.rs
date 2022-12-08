use std::env::current_dir;

use cosmwasm_schema::{export_schema, schema_for, write_api};

use ibc_reflect_send::ibc_msg::PacketMsg;
use ibc_reflect_send::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    // Clear & write standard API
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }

    // Schemas for inter-contract communication
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    export_schema(&schema_for!(PacketMsg), &out_dir);
}
