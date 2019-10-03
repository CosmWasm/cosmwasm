use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize)]
pub struct BlockInfo {
    pub block_height: i64,
    // block_time is RFC3339 encoded timestamp
    pub block_time: String,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct MessageInfo {
    pub signer: String,
    pub sent_funds: Vec<SendAmount>,
}

#[derive(Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub balance: Vec<SendAmount>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SendAmount {
    pub denom: String,
    pub amount: String,
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
pub enum ContractResult {
    #[serde(rename = "msgs")]
    Msgs(Vec<CosmosMsg>),
    #[serde(rename = "error")]
    Error(String),
}

// just set signer, sent funds, and balance - rest given defaults
// this is intended for use in testcode only
pub fn mock_params(signer: &str, sent: &[SendAmount], balance: &[SendAmount]) -> Params {
    Params{
        block: BlockInfo{
            block_height: 12345,
            block_time: "2020-01-08T12:34:56Z".to_string(),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo{
            signer: signer.to_string(),
            sent_funds: sent.to_vec(),
        },
        contract: ContractInfo{
            address: "cosmos2contract".to_string(),
            balance: balance.to_vec()
        }
    }
}

// coin is a shortcut constructor for a set of one denomination of coins
pub fn coin(amount: &str, denom: &str) -> Vec<SendAmount> {
    vec![SendAmount {
        amount: amount.to_string(),
        denom: denom.to_string(),
    }]
}