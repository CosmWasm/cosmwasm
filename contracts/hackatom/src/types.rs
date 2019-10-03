use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub contract_address: String,
    pub sender: String,
    pub sent_funds: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CosmosMsg {
    #[serde(rename = "cosmos-sdk/MsgSend")]
    SendTx {
        from_address: String,
        to_address: String,
        amount: Vec<SendAmount>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct SendAmount {
    pub denom: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize)]
pub enum ContractResult {
    #[serde(rename = "msgs")]
    Msgs(Vec<CosmosMsg>),
    #[serde(rename = "error")]
    Error(String),
}