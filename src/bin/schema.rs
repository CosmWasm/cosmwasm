use schemars::{schema_for};

use cosmwasm::types::{ContractResult, CosmosMsg, Params};

fn main() {
    println!("*** Params ***");
    let schema = schema_for!(Params);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    println!("");

    println!("*** ContractResult ***");
    let schema = schema_for!(ContractResult);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    println!("");

    println!("*** CosmosMsg ***");
    let schema = schema_for!(CosmosMsg);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    println!("");
}