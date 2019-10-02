use crate::{CosmosMsg, InitParams, SendAmount, SendParams};
use crate::storage::Storage;

use failure::{bail, Error};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str, to_vec};

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

pub fn init<T: Storage>(mut store: T, params: InitParams) -> Result<Vec<CosmosMsg>, Error> {
    let msg: RegenInitMsg = from_str(params.msg.get())?;

    store.set_state(to_vec(&RegenState {
        verifier: msg.verifier,
        beneficiary: msg.beneficiary,
        payout: params.sent_funds,
        funder: params.sender
    })?);

    Ok(Vec::new())
}

pub fn send<T:Storage>(mut store: T, params: SendParams) -> Result<Vec<CosmosMsg>, Error> {
    let data = store.get_state();
    let mut state: RegenState = match data {
        Some(v) => from_slice(&v)?,
        None => { bail!("Not initialized") }
    };
    let funds = state.payout + params.sent_funds;
    state.payout = 0;
    store.set_state(to_vec(&state)?);

    if params.sender == state.verifier {
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
