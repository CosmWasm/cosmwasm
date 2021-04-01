use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::CosmosMsg;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema_with_title(&mut schema_for!(CosmosMsg), &out_dir, "CosmosMsg");
}
