use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub block: BlockInfo,
    pub message: MessageInfo,
    pub contract: ContractInfo,
}

#[derive(Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: i64,
    // time is seconds since epoch begin (Jan. 1, 1970)
    pub time: i64,
    pub chain_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct MessageInfo {
    pub signer: String,
    pub sent_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub balance: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug, PartialEq)]
pub struct Coin {
    pub denom: String,
    pub amount: String,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CosmosMsg {
    // this moves tokens in the underlying sdk
    Send {
        from_address: String,
        to_address: String,
        amount: Vec<Coin>,
    },
    // this dispatches a call to another contract at a known address (with known ABI)
    // msg is the json-encoded HandleMsg struct
    Contract {
        contract_addr: String,
        msg: String,
    },
    // this should never be created here, just passed in from the user and later dispatched
    Opaque {
        data: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Default)]
#[derive(Debug, PartialEq)]
pub struct Response {
    // let's make the positive case a struct, it contrains Msg: {...}, but also Data, Log, maybe later Events, etc.
    pub messages: Vec<CosmosMsg>,
    pub log: Option<String>,
    pub data: Option<String>,
}

// just set signer, sent funds, and balance - rest given defaults
// this is intended for use in testcode only
pub fn mock_params(signer: &str, sent: &[Coin], balance: &[Coin]) -> Params {
    Params {
        block: BlockInfo {
            height: 12345,
            time: 1571797419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            signer: signer.to_string(),
            sent_funds: sent.to_vec(),
        },
        contract: ContractInfo {
            address: "cosmos2contract".to_string(),
            balance: balance.to_vec(),
        },
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
        let back = from_slice(&bin).expect("decode contract result");
        assert_eq!(fail, back);
    }

    #[test]
    fn can_deser_ok_result() {
        let send = Response {
            messages: vec![CosmosMsg::Send {
                from_address: "me".to_string(),
                to_address: "you".to_string(),
                amount: coin("1015", "earth"),
            }],
            log: Some("released funds!".to_string()),
            data: None,
        };
        let bin = to_vec(&send).expect("encode contract result");
        use std::str::from_utf8;
        println!("msg {}", from_utf8(&bin).unwrap());
        let back = from_slice(&bin).expect("decode contract result");
        assert_eq!(send, back);
    }

}