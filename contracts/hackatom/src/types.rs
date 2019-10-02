use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Serialize, Deserialize)]
pub struct SendParams<'a> {
    pub contract_address: String,
    pub sender: String,
    #[serde(borrow)]
    pub msg: &'a RawValue,
    pub sent_funds: u64,
}

#[derive(Serialize, Deserialize)]
pub struct RegenSendMsg {}

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

#[derive(Serialize, Deserialize)]
pub struct InitParams<'a> {
    pub contract_address: String,
    pub sender: String,
    #[serde(borrow)]
    pub msg: &'a RawValue,
    pub sent_funds: u64,
}
