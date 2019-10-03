use crate::imports::Storage;
use crate::types::{CosmosMsg, Params};

use failure::{bail, format_err, Error};
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
    funder: String,
}

#[derive(Serialize, Deserialize)]
struct RegenSendMsg {}

static CONFIG_KEY: &[u8] = b"config";

pub fn init<T: Storage>(
    store: &mut T,
    params: Params,
    msg: Vec<u8>,
) -> Result<Vec<CosmosMsg>, Error> {
    let msg: RegenInitMsg = from_slice(&msg)?;
    store.set(
        CONFIG_KEY,
        &to_vec(&RegenState {
            verifier: msg.verifier,
            beneficiary: msg.beneficiary,
            funder: params.message.signer,
        })?,
    );
    Ok(Vec::new())
}

pub fn send<T: Storage>(
    store: &mut T,
    params: Params,
    _: Vec<u8>,
) -> Result<Vec<CosmosMsg>, Error> {
    let data = store
        .get(CONFIG_KEY)
        .ok_or(format_err!("not initialized"))?;
    let state: RegenState = from_slice(&data)?;

    if params.message.signer == state.verifier {
        Ok(vec![CosmosMsg::SendTx {
            from_address: params.contract.address,
            to_address: state.beneficiary,
            amount: params.contract.balance,
        }])
    } else {
        bail!("Unauthorized")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imports::MockStorage;
    use crate::types::{coin, mock_params};

    #[test]
    fn proper_initialization() {
        let mut store = MockStorage::new();
        let msg = serde_json::to_vec(&RegenInitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let res = init(&mut store, params, msg).unwrap();
        assert_eq!(0, res.len());

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }

    #[test]
    fn fails_on_bad_init() {
        let mut store = MockStorage::new();
        let bad_msg = b"{}".to_vec();
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let res = init(&mut store, params, bad_msg);
        assert_eq!(true, res.is_err());
    }

    #[test]
    fn proper_send() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = serde_json::to_vec(&RegenInitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.len());

        // beneficiary can release it
        let send_params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
        let send_res = send(&mut store, send_params, Vec::new()).unwrap();
        assert_eq!(1, send_res.len());
        let msg = send_res.get(0).expect("no message");
        match &msg {
            CosmosMsg::SendTx {
                from_address,
                to_address,
                amount,
            } => {
                assert_eq!("cosmos2contract", from_address);
                assert_eq!("benefits", to_address);
                assert_eq!(1, amount.len());
                let coin = amount.get(0).expect("No coin");
                assert_eq!(coin.denom, "earth");
                assert_eq!(coin.amount, "1015");
            }
        }

        // it worked, let's check the state
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }

    #[test]
    fn failed_send() {
        let mut store = MockStorage::new();

        // initialize the store
        let init_msg = serde_json::to_vec(&RegenInitMsg {
            verifier: String::from("verifies"),
            beneficiary: String::from("benefits"),
        })
        .unwrap();
        let init_params = mock_params("creator", &coin("1000", "earth"), &coin("1000", "earth"));
        let init_res = init(&mut store, init_params, init_msg).unwrap();
        assert_eq!(0, init_res.len());

        // beneficiary can release it
        let send_params = mock_params("benefits", &[], &coin("1000", "earth"));
        let send_res = send(&mut store, send_params, Vec::new());
        assert!(send_res.is_err());

        // state should not change
        let data = store.get(CONFIG_KEY).expect("no data stored");
        let state: RegenState = from_slice(&data).unwrap();
        assert_eq!(state.verifier, String::from("verifies"));
        assert_eq!(state.beneficiary, String::from("benefits"));
        assert_eq!(state.funder, String::from("creator"));
    }
}
