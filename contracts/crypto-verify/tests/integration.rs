//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatim,
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see init/handle(deps.as_mut(), ...) you must replace it with init/handle(&mut deps, ...)
//! 5. Anywhere you see query(deps.as_ref(), ...) you must replace it with query(&mut deps, ...)
//! (Use cosmwasm_vm::testing::{init, handle, query}, instead of the contract variants).

use cosmwasm_vm::testing::{
    handle, init, mock_env, mock_info, mock_instance, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{from_slice, Instance};

use cosmwasm_std::{attr, Binary, Response};

use crypto_verify::msg::{HandleMsg, InitMsg, ListVerificationsResponse, QueryMsg};

// Output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/crypto_verify.wasm");

const CREATOR: &str = "creator";
const SENDER: &str = "sender";

const HASH_HEX: &str = "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
const SIGNATURE_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
const PUBLIC_KEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

fn setup() -> Instance<MockApi, MockStorage, MockQuerier> {
    let mut deps = mock_instance(WASM, &[]);
    let msg = InitMsg {};
    let info = mock_info(CREATOR, &[]);
    let res: Response = init(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

#[test]
fn init_works() {
    setup();
}

#[test]
fn verify_works() {
    let mut deps = setup();

    let hash = hex::decode(HASH_HEX).unwrap();
    let signature = hex::decode(SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(PUBLIC_KEY_HEX).unwrap();

    let verify_msg = HandleMsg::VerifySignature {
        message_hash: Binary(hash),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };
    let res: Response = handle(&mut deps, mock_env(), mock_info(SENDER, &[]), verify_msg).unwrap();

    assert_eq!(
        res,
        Response {
            messages: vec![],
            attributes: vec![attr("action", "handle_verify")],
            data: Some(Binary(vec![1]))
        }
    );
}

#[test]
fn verify_fails() {
    let mut deps = setup();

    let mut hash = hex::decode(HASH_HEX).unwrap();
    // alter hash
    hash[0] ^= 0x01;
    let signature = hex::decode(SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(PUBLIC_KEY_HEX).unwrap();

    let verify_msg = HandleMsg::VerifySignature {
        message_hash: Binary(hash),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };
    let res: Response = handle(&mut deps, mock_env(), mock_info(SENDER, &[]), verify_msg).unwrap();

    assert_eq!(
        res,
        Response {
            messages: vec![],
            attributes: vec![attr("action", "handle_verify")],
            data: Some(Binary(vec![0]))
        }
    );
}

#[test]
#[should_panic(expected = "empty")]
fn verify_panics() {
    let mut deps = setup();

    let hash = hex::decode(HASH_HEX).unwrap();
    let signature = hex::decode(SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = HandleMsg::VerifySignature {
        message_hash: Binary(hash),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };
    let _: Response = handle(&mut deps, mock_env(), mock_info(SENDER, &[]), verify_msg).unwrap();
}

#[test]
fn query_works() {
    let mut deps = setup();

    let query_msg = QueryMsg::ListVerificationSchemes {};

    let raw = query(&mut deps, mock_env(), query_msg).unwrap();
    let res: ListVerificationsResponse = from_slice(&raw).unwrap();

    assert_eq!(
        res,
        ListVerificationsResponse {
            verification_schemes: vec!["secp256k1".into()]
        }
    );
}
