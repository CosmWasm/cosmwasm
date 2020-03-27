use std::env::current_dir;
use std::fs::{create_dir_all, write};
use std::path::PathBuf;

use schemars::{schema::RootSchema, schema_for};

use cosmwasm_std::{ContractResult, CosmosMsg, Env};

fn main() {
    let mut pwd = current_dir().unwrap();
    pwd.push("schema");
    create_dir_all(&pwd).unwrap();

    let schema = schema_for!(Env);
    export_schema(&schema, &pwd, "env.json");

    let schema = schema_for!(CosmosMsg);
    export_schema(&schema, &pwd, "cosmos_msg.json");

    let schema = schema_for!(ContractResult);
    export_schema(&schema, &pwd, "contract_result.json");

    let schema = schema_for!(ContractResult);
    export_schema(&schema, &pwd, "query_result.json");
}

/// Writes schema to file. Overwrites existing file.
/// Panics on any error writing out the schema.
fn export_schema(schema: &RootSchema, out_dir: &PathBuf, file_name: &str) -> () {
    let path = out_dir.join(file_name);
    let json = serde_json::to_string_pretty(schema).unwrap();
    write(&path, json + "\n").unwrap();
    println!("Created {}", path.to_str().unwrap());
}
