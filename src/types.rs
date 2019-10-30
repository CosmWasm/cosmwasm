use prost_derive::{Message};

#[derive(Message)]
pub struct Params {
    #[prost(message, optional, tag="1")]
    pub block: Option<BlockInfo>,
    #[prost(message, optional, tag="2")]
    pub message: Option<MessageInfo>,
    #[prost(message, optional, tag="3")]
    pub contract: Option<ContractInfo>,
}

#[derive(Message)]
pub struct BlockInfo {
    #[prost(int64, tag="1")]
    pub height: i64,
    // time is seconds since epoch begin (Jan. 1, 1970)
    #[prost(int64, tag="2")]
    pub time: i64,
    #[prost(string, tag="3")]
    pub chain_id: String,
}

#[derive(Message)]
pub struct MessageInfo {
    #[prost(string, tag="1")]
    pub signer: String,
    #[prost(message, repeated, tag="2")]
    pub sent_funds: Vec<Coin>,
}

#[derive(Message)]
pub struct ContractInfo {
    #[prost(string, tag="1")]
    pub address: String,
    #[prost(message, repeated, tag="2")]
    pub balance: Vec<Coin>,
}

#[derive(Message, Clone)]
pub struct Coin {
    #[prost(string, tag="1")]
    pub denom: String,
    #[prost(string, tag="2")]
    pub amount: String,
}

#[derive(Message)]
pub struct Msg {
    #[prost(oneof = "CosmosMsg", tags = "1, 2, 3")]
    pub msg: Option<CosmosMsg>,
}

#[derive(prost::Oneof)]
pub enum CosmosMsg {
    #[prost(message, tag = "1")]
    Send(SendMsg),
    #[prost(message, tag = "2")]
    Contract(ContractMsg),
    #[prost(message, tag = "3")]
    Opaque(OpaqueMsg),
}

// this moves tokens in the underlying sdk
#[derive(Message)]
pub struct SendMsg {
    #[prost(string, tag="1")]
    pub from_address: String,
    #[prost(string, tag="2")]
    pub to_address: String,
    #[prost(message, repeated, tag="3")]
    pub amount: Vec<Coin>,
}
// this dispatches a call to another contract at a known address (with known ABI)
// msg is the json-encoded HandleMsg struct
#[derive(Message)]
pub struct ContractMsg {
    #[prost(string, tag="1")]
    pub contract_addr: String,
    #[prost(string, tag="2")]
    pub msg: String,
}
// this should never be created here, just passed in from the user and later dispatched
#[derive(Message)]
pub struct OpaqueMsg {
    #[prost(string, tag="1")]
    pub data: String,
}

#[derive(Message)]
pub struct ContractResult {
    #[prost(oneof = "Result", tags = "1, 2")]
    pub res: Option<Result>,
}


#[derive(prost::Oneof)]
pub enum Result {
    #[prost(message, tag = "1")]
    Ok(Response),
    #[prost(message, tag = "2")]
    Err(String),
}

impl ContractResult {
    // unwrap will panic on err, or give us the real data useful for tests
    pub fn unwrap(self) -> Response {
        match self.res.unwrap() {
            Result::Err(msg) => panic!("Unexpected error: {}", msg),
            Result::Ok(res) => res,
        }
    }

    pub fn is_err(&self) -> bool {
        match self.res.as_ref().unwrap() {
            Result::Err(_) => true,
            _ => false,
        }
    }
}

#[derive(Message)]
pub struct Response {
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    #[prost(message, repeated, tag="1")]
    pub messages: Vec<Msg>,
    #[prost(string, optional, tag="2")]
    pub log: Option<String>,
    #[prost(string, optional, tag="3")]
    pub data: Option<String>,
}

// just set signer, sent funds, and balance - rest given defaults
// this is intended for use in testcode only
pub fn mock_params(signer: &str, sent: &[Coin], balance: &[Coin]) -> Params {
    Params {
        block: Some(BlockInfo {
            height: 12_345,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        }),
        message: Some(MessageInfo {
            signer: signer.to_string(),
            sent_funds: sent.to_vec(),
        }),
        contract: Some(ContractInfo {
            address: "cosmos2contract".to_string(),
            balance: balance.to_vec(),
        }),
    }
}

// coin is a shortcut constructor for a set of one denomination of coins
pub fn coin(amount: &str, denom: &str) -> Vec<Coin> {
    vec![Coin {
        amount: amount.to_string(),
        denom: denom.to_string(),
    }]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prost::{from_slice, to_vec};

    #[test]
    fn can_deser_error_result() {
        let fail = ContractResult{res: Some(Result::Err("foobar".to_string()))};
        let bin = to_vec(&fail).expect("encode contract result");
        println!("error: {}", std::str::from_utf8(&bin).unwrap());
        let _: ContractResult = from_slice(&bin).expect("decode contract result");
        // need Derive Debug and PartialEq for this, removed to save space
//        assert_eq!(fail, back);
    }

    #[test]
    fn can_deser_ok_result() {
        let send = ContractResult{res: Some(Result::Ok(Response {
            messages: vec![Msg{msg: Some(CosmosMsg::Send(SendMsg {
                from_address: "me".to_string(),
                to_address: "you".to_string(),
                amount: coin("1015", "earth"),
            }))}],
            log: Some("released funds!".to_string()),
            data: None,
        }))};
        let bin = to_vec(&send).expect("encode contract result");
        println!("ok: {}", std::str::from_utf8(&bin).unwrap());
        let _: ContractResult = from_slice(&bin).expect("decode contract result");
        // need Derive Debug and PartialEq for this, removed to save space
//        assert_eq!(send, back);
    }
}
