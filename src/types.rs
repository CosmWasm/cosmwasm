use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct HumanAddr(pub String);

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct CanonicalAddr(pub Vec<u8>);

impl HumanAddr {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn len(&self) -> usize { self.0.len() }
}

impl fmt::Display for HumanAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl CanonicalAddr {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn len(&self) -> usize { self.0.len() }
}

// upper-case hex
impl fmt::Display for CanonicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:X} ", byte)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Params {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct BlockInfo {
    pub height: i64,
    // time is seconds since epoch begin (Jan. 1, 1970)
    pub time: i64,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct MessageInfo {
    pub signer: CanonicalAddr,
    // go likes to return null for empty array, make sure we can parse it (use option)
    pub sent_funds: Option<Vec<Coin>>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    pub address: CanonicalAddr,
    // go likes to return null for empty array, make sure we can parse it (use option)
    pub balance: Option<Vec<Coin>>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CosmosMsg {
    // this moves tokens in the underlying sdk
    Send {
        from_address: HumanAddr,
        to_address: HumanAddr,
        amount: Vec<Coin>,
    },
    // this dispatches a call to another contract at a known address (with known ABI)
    // msg is the json-encoded HandleMsg struct
    Contract {
        contract_addr: HumanAddr,
        msg: String,
        send: Vec<Coin>,
    },
    // this should never be created here, just passed in from the user and later dispatched
    Opaque {
        data: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContractResult {
    Ok(Response),
    Err(String),
}

impl ContractResult {
    // unwrap will panic on err, or give us the real data useful for tests
    pub fn unwrap(self) -> Response {
        match self {
            ContractResult::Err(msg) => panic!("Unexpected error: {}", msg),
            ContractResult::Ok(res) => res,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            ContractResult::Err(_) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct Response {
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg>,
    pub log: Option<String>,
    pub data: Option<String>,
}

// RawQuery is a default query that can easily be supported by all contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawQuery {
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryResult {
    Ok(Vec<u8>),
    Err(String),
}

impl QueryResult {
    // unwrap will panic on err, or give us the real data useful for tests
    pub fn unwrap(self) -> Vec<u8> {
        match self {
            QueryResult::Err(msg) => panic!("Unexpected error: {}", msg),
            QueryResult::Ok(res) => res,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            QueryResult::Err(_) => true,
            _ => false,
        }
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
    use crate::serde::{from_slice, to_vec};

    #[test]
    fn can_deser_error_result() {
        let fail = ContractResult::Err("foobar".to_string());
        let bin = to_vec(&fail).expect("encode contract result");
        println!("error: {}", std::str::from_utf8(&bin).unwrap());
        let back: ContractResult = from_slice(&bin).expect("decode contract result");
        assert_eq!(fail, back);
    }

    #[test]
    fn can_deser_ok_result() {
        let send = ContractResult::Ok(Response {
            messages: vec![CosmosMsg::Send {
                from_address: HumanAddr("me".to_string()),
                to_address: HumanAddr("you".to_string()),
                amount: coin("1015", "earth"),
            }],
            log: Some("released funds!".to_string()),
            data: None,
        });
        let bin = to_vec(&send).expect("encode contract result");
        println!("ok: {}", std::str::from_utf8(&bin).unwrap());
        let back: ContractResult = from_slice(&bin).expect("decode contract result");
        assert_eq!(send, back);
    }
}
