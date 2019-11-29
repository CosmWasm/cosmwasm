use std::env::current_dir;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use schemars::{schema_for, schema::RootSchema};

use cosmwasm::types::{ContractResult, CosmosMsg, Params};

fn main() {
    let mut pwd = current_dir().unwrap();
    pwd.push("schema");
    create_dir_all(&pwd).unwrap();

    let schema = schema_for!(Params);
    export_schema(&schema, &pwd, "params.json");

    let schema = schema_for!(CosmosMsg);
    export_schema(&schema, &pwd, "cosmos_msg.json");

    let schema = schema_for!(ContractResult);
    export_schema(&schema, &pwd, "contract_result.json");

    let schema = schema_for!(ContractResult);
    export_schema(&schema, &pwd, "query_result.json");
}

// panics if
fn export_schema(schema: &RootSchema, dir: &PathBuf, name: &str) -> () {
    let path = dir.join(name);
    let json = serde_json::to_string_pretty(schema).unwrap();
    write(&path, json.as_bytes()).unwrap();
    println!("{}", path.to_str().unwrap());
}