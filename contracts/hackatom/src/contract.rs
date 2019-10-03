use crate::types::{CosmosMsg, Params, SendAmount};
use crate::imports::Storage;

use failure::{bail, Error};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};

#[derive(Serialize, Deserialize)]
struct RegenInitMsg {
    verifier: String,
    beneficiary: String,
}

#[derive(Serialize, Deserialize)]
struct RegenState {
    verifier: String,
    beneficiary: String,
    payout: u64,
    funder: String,
}

#[derive(Serialize, Deserialize)]
struct RegenSendMsg {}

static CONFIG_KEY: &[u8] = b"config";

pub fn init<T: Storage>(store: &mut T, params: Params, msg: Vec<u8>) -> Result<Vec<CosmosMsg>, Error> {
    let msg: RegenInitMsg = from_slice(&msg)?;
    store.set(CONFIG_KEY, &to_vec(&RegenState {
        verifier: msg.verifier,
        beneficiary: msg.beneficiary,
        payout: params.sent_funds,
        funder: params.sender
    })?);

    Ok(Vec::new())
}

pub fn send<T:Storage>(store: &mut T, params: Params, _: Vec<u8>) -> Result<Vec<CosmosMsg>, Error> {
    let data = store.get(CONFIG_KEY);
    let mut state: RegenState = match data {
        Some(v) => from_slice(&v)?,
        None => { bail!("Not initialized") }
    };

    if params.sender == state.verifier {
        let funds = state.payout + params.sent_funds;
        state.payout = 0;
        store.set(CONFIG_KEY, &to_vec(&state)?);
        Ok(vec![CosmosMsg::SendTx {
            from_address: params.contract_address,
            to_address: state.beneficiary,
            amount: vec![SendAmount {
                denom: "earth".into(),
                amount: funds.to_string(),
            }],
        }])
    } else {
        bail!("Unauthorized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imports::{MockStorage};

    #[test]
    fn proper_initialization() {
        let mut store = MockStorage::new();
        let msg = serde_json::to_vec(&RegenInitMsg{
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        }).unwrap();
        let params = Params {
            contract_address: String::from("contract"),
            sender: String::from("creator"),
            sent_funds: 1000,
        };
        let res = init(&mut store, params, msg).unwrap();
        assert_eq!(0, res.len());

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.payout, 1000);
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }

    #[test]
    fn fails_on_bad_init() {
        let mut store = MockStorage::new();
        let bad_msg = b"{}".to_vec();
        let params = Params {
            contract_address: String::from("contract"),
            sender: String::from("creator"),
            sent_funds: 1000,
        };
        let res = init(&mut store, params, bad_msg);
        if let Ok(_) = res {
            assert!(false);
        }
    }

    #[test]
    fn proper_send() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = serde_json::to_vec(&RegenInitMsg{
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        }).unwrap();
        let init_params = Params {
            contract_address: String::from("contract"),
            sender: String::from("creator"),
            sent_funds: 1000,
        };
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.len());

        // beneficiary can release it
        let send_params = Params {
            contract_address: String::from("contract"),
            sender: String::from("verifies"),
            sent_funds: 15,
        };
        let send_res = send(&mut store, send_params, Vec::new()).unwrap();
        assert_eq!(1, send_res.len());
        let msg = send_res.get(0).expect("no message");
        match &msg {
            CosmosMsg::SendTx{from_address, to_address, amount} => {
                assert_eq!("contract", from_address);
                assert_eq!("benefits", to_address);
                assert_eq!(1, amount.len());
                match amount.get(0) {
                    Some(coin) => {
                        assert_eq!(coin.denom, "earth");
                        assert_eq!(coin.amount, "1015");
                    },
                    None => panic!("No coin"),
                }
            },
        }

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.payout, 0);
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }

    #[test]
    fn failed_send() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = serde_json::to_vec(&RegenInitMsg{
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        }).unwrap();
        let init_params = Params {
            contract_address: String::from("contract"),
            sender: String::from("creator"),
            sent_funds: 1000,
        };
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.len());

        // beneficiary can release it
        let send_params = Params {
            contract_address: String::from("contract"),
            sender: String::from("benefits"),
            sent_funds: 0,
        };
        let send_res = send(&mut store, send_params, Vec::new());
        assert!(send_res.is_err());

        // state should not change
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.payout, 1000);
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }

}