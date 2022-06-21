use std::collections::HashMap;

use cosmwasm_schema::{schema_for, Api, IDL_VERSION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub cap: u128,
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint { amount: u128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { account: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    SetAdmin { new_admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub admin: String,
    pub cap: u128,
}

#[test]
fn test_basic_structure() {
    let api_str = Api {
        instantiate: schema_for!(InstantiateMsg),
        execute: schema_for!(ExecuteMsg),
        query: schema_for!(QueryMsg),
        migrate: Some(schema_for!(MigrateMsg)),
        sudo: Some(schema_for!(SudoMsg)),
        responses: [("balance".to_string(), schema_for!(u128))]
            .into_iter()
            .collect(),
    }
    .render()
    .to_string()
    .unwrap();

    let api_json: HashMap<String, Value> = serde_json::from_str(&api_str).unwrap();
    assert_eq!(api_json.get("idl_version").unwrap(), IDL_VERSION);
    assert_eq!(
        api_json.get("instantiate").unwrap().get("title").unwrap(),
        "InstantiateMsg"
    );
    assert_eq!(
        api_json.get("execute").unwrap().get("title").unwrap(),
        "ExecuteMsg"
    );
    assert_eq!(
        api_json.get("query").unwrap().get("title").unwrap(),
        "QueryMsg"
    );
    assert_eq!(
        api_json.get("migrate").unwrap().get("title").unwrap(),
        "MigrateMsg"
    );
    assert_eq!(
        api_json.get("sudo").unwrap().get("title").unwrap(),
        "SudoMsg"
    );
}

#[test]
fn test_query_responses() {
    let api_str = Api {
        instantiate: schema_for!(InstantiateMsg),
        execute: schema_for!(ExecuteMsg),
        query: schema_for!(QueryMsg),
        migrate: None,
        sudo: None,
        responses: [("balance".to_string(), schema_for!(u128))]
            .into_iter()
            .collect(),
    }
    .render()
    .to_string()
    .unwrap();

    let api: Value = serde_json::from_str(&api_str).unwrap();
    let queries = api
        .get("query")
        .unwrap()
        .get("oneOf")
        .unwrap()
        .as_array()
        .unwrap();

    // Find the "balance" query in the queries schema
    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].get("required").unwrap().get(0).unwrap(),
        "balance"
    );

    // Find the "balance" query in responses
    api.get("responses").unwrap().get("balance").unwrap();
}
