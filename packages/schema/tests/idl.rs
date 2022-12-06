use std::collections::HashMap;

use cosmwasm_schema::{cw_serde, generate_api, QueryResponses, IDL_VERSION};
use serde_json::Value;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub cap: u128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint { amount: u128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(u128)]
    Balance { account: String },
}

#[cw_serde]
pub enum SudoMsg {
    SetAdmin { new_admin: String },
}

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgWithGenerics<T> {
    #[returns(u128)]
    QueryData { data: T },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsgWithGenericsAndTraitBounds<T: std::fmt::Debug>
where
    T: PartialEq,
{
    #[returns(u128)]
    QueryData { data: T },
}

#[cw_serde]
#[derive(QueryResponses)]
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

#[cw_serde]
#[derive(QueryResponses)]
#[serde(untagged)]
#[query_responses(nested)]
pub enum NestedQueryMsg {
    Query(QueryMsg),
    Sub(SubQueryMsg1),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum SubQueryMsg1 {
    #[returns(u128)]
    Variant1 { test: String },
}

#[test]
fn test_nested_query_responses() {
    generate_api! {
        instantiate: InstantiateMsg,
        query: NestedQueryMsg,
    }
    .render()
    .to_string()
    .unwrap();
}

#[cw_serde]
#[derive(QueryResponses)]
#[serde(untagged)]
#[query_responses(nested)]
pub enum NestedQueryMsgGenerics<T, U> {
    /// doc comment
    Query(T),
    Sub(U),
}

#[test]
fn test_nested_query_responses_with_generics() {
    generate_api! {
        instantiate: InstantiateMsg,
        query: NestedQueryMsgGenerics<QueryMsg, SubQueryMsg1>,
    }
    .render()
    .to_string()
    .unwrap();
}

#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg2 {
    #[returns(u128)]
    Balance {},
}

#[cw_serde]
#[derive(QueryResponses)]
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
