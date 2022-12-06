use std::collections::HashMap;

use cosmwasm_schema::{generate_api, QueryResponses, IDL_VERSION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: String,
    pub cap: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint { amount: u128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    #[returns(u128)]
    Balance { account: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    SetAdmin { new_admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {
    pub admin: String,
    pub cap: u128,
}

#[test]
fn test_basic_structure() {
    let api_str = generate_api! {
        name: "test",
        version: "0.1.0",
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        sudo: SudoMsg,
        migrate: MigrateMsg,
    }
    .render()
    .to_string()
    .unwrap();

    let api_json: HashMap<String, Value> = serde_json::from_str(&api_str).unwrap();
    assert_eq!(api_json.get("idl_version").unwrap(), IDL_VERSION);
    assert_eq!(api_json.get("contract_name").unwrap(), "test");
    assert_eq!(api_json.get("contract_version").unwrap(), "0.1.0");
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

// Test to reproduce https://github.com/CosmWasm/cosmwasm/issues/1527
#[test]
fn generate_api_works_when_only_types_are_imported() {
    mod my_api_generator {
        // Note super::QueryResponses is not imported in that case.
        use super::generate_api;
        use super::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};

        pub fn generate() {
            let _api_str = generate_api! {
                name: "test",
                version: "0.1.0",
                instantiate: InstantiateMsg,
                query: QueryMsg,
                execute: ExecuteMsg,
                sudo: SudoMsg,
                migrate: MigrateMsg,
            }
            .render()
            .to_string()
            .unwrap();
        }
    }
    my_api_generator::generate();
}

#[test]
fn test_query_responses() {
    let api_str = generate_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgWithGenerics<T> {
    #[returns(u128)]
    QueryData { data: T },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgWithGenericsAndTraitBounds<T: std::fmt::Debug>
where
    T: PartialEq,
{
    #[returns(u128)]
    QueryData { data: T },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgWithGenericsAndDefaultType<T = u128> {
    #[returns(u128)]
    QueryData { data: T },
}

#[test]
fn test_query_responses_generics() {
    let api_str = generate_api! {
        instantiate: InstantiateMsg,
        query: QueryMsgWithGenerics<u32>,
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

    // Find the "query_data" query in the queries schema
    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].get("required").unwrap().get(0).unwrap(),
        "query_data"
    );

    // Find the "query_data" query in responses
    api.get("responses").unwrap().get("query_data").unwrap();
}

#[test]
fn test_query_responses_generics_and_trait_bounds() {
    let api_str = generate_api! {
        instantiate: InstantiateMsg,
        query: QueryMsgWithGenericsAndTraitBounds<u32>,
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

    // Find the "query_data" query in the queries schema
    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].get("required").unwrap().get(0).unwrap(),
        "query_data"
    );

    // Find the "query_data" query in responses
    api.get("responses").unwrap().get("query_data").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(untagged)]
#[query_responses(nested)]
pub enum NestedQueryMsg {
    Query(QueryMsg),
    Sub(SubQueryMsg1),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum SubQueryMsg1 {
    #[returns(u128)]
    Variant1 { test: String },
}

#[test]
fn test_nested_query_responses() {
    let api_str = generate_api! {
        instantiate: InstantiateMsg,
        query: NestedQueryMsg,
    }
    .render()
    .to_string()
    .unwrap();

    let api: Value = serde_json::from_str(&api_str).unwrap();
    let queries = api
        .get("query")
        .unwrap()
        .get("anyOf")
        .unwrap()
        .as_array()
        .unwrap();
    let definitions = api.get("query").unwrap().get("definitions").unwrap();

    // Find the subqueries
    assert_eq!(queries.len(), 2);
    assert_eq!(
        queries[0].get("$ref").unwrap().as_str().unwrap(),
        "#/definitions/QueryMsg"
    );
    assert_eq!(
        queries[1].get("$ref").unwrap().as_str().unwrap(),
        "#/definitions/SubQueryMsg1"
    );
    let query_msg_queries = definitions
        .get("QueryMsg")
        .unwrap()
        .get("oneOf")
        .unwrap()
        .as_array()
        .unwrap();
    let sub_query_msg_queries = definitions
        .get("SubQueryMsg1")
        .unwrap()
        .get("oneOf")
        .unwrap()
        .as_array()
        .unwrap();

    // Find "balance" and "variant1" queries in the query schema
    assert_eq!(
        query_msg_queries[0]
            .get("required")
            .unwrap()
            .get(0)
            .unwrap(),
        "balance"
    );
    assert_eq!(
        sub_query_msg_queries[0]
            .get("required")
            .unwrap()
            .get(0)
            .unwrap(),
        "variant1"
    );

    // Find "balance" and "variant1" queries in responses
    api.get("responses").unwrap().get("balance").unwrap();
    api.get("responses").unwrap().get("variant1").unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
enum QueryMsg2 {
    #[returns(u128)]
    Balance {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, QueryResponses)]
#[query_responses(nested)]
enum NestedNameCollision {
    Q1(QueryMsg),
    Q2(QueryMsg2),
}

#[test]
#[should_panic = "name collision in subqueries for idl::NestedNameCollision"]
fn nested_name_collision_caught() {
    generate_api! {
        instantiate: InstantiateMsg,
        query: NestedNameCollision,
    };
}
