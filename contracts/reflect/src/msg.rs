use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, CosmosMsg, CustomQuery, QueryRequest, SubMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ReflectMsg { msgs: Vec<CosmosMsg<CustomMsg>> },
    ReflectSubMsg { msgs: Vec<SubMsg<CustomMsg>> },
    ChangeOwner { owner: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Owner {},
    /// This will call out to SpecialQuery::Capitalized
    Capitalized {
        text: String,
    },
    /// Queries the blockchain and returns the result untouched
    Chain {
        request: QueryRequest<SpecialQuery>,
    },
    /// Queries another contract and returns the data
    Raw {
        contract: String,
        key: Binary,
    },
    /// If there was a previous ReflectSubMsg with this ID, returns cosmwasm_std::Reply
    SubMsgResult {
        id: u64,
    },
}

// We define a custom struct for each query response

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CapitalizedResponse {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ChainResponse {
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RawResponse {
    /// The returned value of the raw query. Empty data can be the
    /// result of a non-existent key or an empty value. We cannot
    /// differentiate those two cases in cross contract queries.
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// CustomMsg is an override of CosmosMsg::Custom to show this works and can be extended in the contract
pub enum CustomMsg {
    Debug(String),
    Raw(Binary),
}

impl cosmwasm_std::CustomMsg for CustomMsg {}

impl From<CustomMsg> for CosmosMsg<CustomMsg> {
    fn from(original: CustomMsg) -> Self {
        CosmosMsg::Custom(original)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// An implementation of QueryRequest::Custom to show this works and can be extended in the contract
pub enum SpecialQuery {
    Ping {},
    Capitalized { text: String },
}

impl CustomQuery for SpecialQuery {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// The response data for all `SpecialQuery`s
pub struct SpecialResponse {
    pub msg: String,
}
