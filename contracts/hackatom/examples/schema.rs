use std::env::current_dir;
use std::fs::{create_dir_all, write};

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, Api, QueryResponses};
use cosmwasm_std::BalanceResponse;

use hackatom::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg, VerifierResponse};
use hackatom::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // messages
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);
    export_schema(&schema_for!(SudoMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(VerifierResponse), &out_dir);
    export_schema(&schema_for!(BalanceResponse), &out_dir);

    // state
    export_schema(&schema_for!(State), &out_dir);

    let contract_name = env!("CARGO_PKG_NAME");
    let contract_version = env!("CARGO_PKG_VERSION");

    // The new IDL
    let path = out_dir.join(format!("{}.json", contract_name));
    let api = Api {
        contract_name: contract_name.to_string(),
        contract_version: contract_version.to_string(),
        instantiate: schema_for!(InstantiateMsg),
        execute: Some(schema_for!(ExecuteMsg)),
        query: Some(schema_for!(QueryMsg)),
        migrate: Some(schema_for!(MigrateMsg)),
        sudo: Some(schema_for!(SudoMsg)),
        responses: Some(QueryMsg::response_schemas().unwrap()),
    }
    .render();
    let json = api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());
}
