use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::{CosmosMsg, Empty, QueryRequest, Timestamp};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(Timestamp), &out_dir);
    export_schema_with_title(&schema_for!(CosmosMsg), &out_dir, "CosmosMsg");
    export_schema_with_title(&schema_for!(QueryRequest<Empty>), &out_dir, "QueryRequest");
}
