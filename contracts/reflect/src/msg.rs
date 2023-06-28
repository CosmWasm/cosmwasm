use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CosmosMsg, CustomQuery, QueryRequest, SubMsg};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ReflectMsg { msgs: Vec<CosmosMsg<CustomMsg>> },
    ReflectSubMsg { msgs: Vec<SubMsg<CustomMsg>> },
    ChangeOwner { owner: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OwnerResponse)]
    Owner {},
    /// This will call out to SpecialQuery::Capitalized
    #[returns(CapitalizedResponse)]
    Capitalized { text: String },
    /// Queries the blockchain and returns the result untouched
    #[returns(ChainResponse)]
    Chain { request: QueryRequest<SpecialQuery> },
    /// Queries another contract and returns the data
    #[returns(RawResponse)]
    Raw { contract: String, key: Binary },
    /// If there was a previous ReflectSubMsg with this ID, returns cosmwasm_std::Reply
    #[returns(cosmwasm_std::Reply)]
    SubMsgResult { id: u64 },
}

// We define a custom struct for each query response

#[cw_serde]
pub struct OwnerResponse {
    pub owner: String,
}

#[cw_serde]
pub struct CapitalizedResponse {
    pub text: String,
}

#[cw_serde]
pub struct ChainResponse {
    pub data: Binary,
}

#[cw_serde]
pub struct RawResponse {
    /// The returned value of the raw query. Empty data can be the
    /// result of a non-existent key or an empty value. We cannot
    /// differentiate those two cases in cross contract queries.
    pub data: Binary,
}

#[cw_serde]
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

#[cw_serde]
/// An implementation of QueryRequest::Custom to show this works and can be extended in the contract
pub enum SpecialQuery {
    Ping {},
    Capitalized { text: String },
}

impl CustomQuery for SpecialQuery {}

#[cw_serde]
/// The response data for all `SpecialQuery`s
pub struct SpecialResponse {
    pub msg: String,
}
