use cosmwasm_schema::{schema_for, Api};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempfile::tempfile;

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
pub enum ContractError {
    Invalid {},
}

#[test]
fn test() {
    let file = tempfile().unwrap();

    let api = Api {
        instantiate: schema_for!(InstantiateMsg),
        execute: schema_for!(ExecuteMsg),
        query: schema_for!(QueryMsg),
        migrate: None,
        sudo: None,
        error: schema_for!(ContractError),
        responses: [("balance".to_string(), schema_for!(u128))]
            .into_iter()
            .collect(),
    }
    .render();
    let _api = serde_json::to_writer_pretty(file, &api).unwrap();
}
