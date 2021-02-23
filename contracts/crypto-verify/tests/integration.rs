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
    init, mock_env, mock_info, mock_instance, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{from_slice, Instance};

use cosmwasm_std::{Binary, Response};

use crypto_verify::msg::{InitMsg, ListVerificationsResponse, QueryMsg, VerifyResponse};

// Output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/crypto_verify.wasm");

const CREATOR: &str = "creator";

const SECP256K1_MESSAGE_HEX: &str = "5c868fedb8026979ebd26f1ba07c27eedf4ff6d10443505a96ecaf21ba8c4f0937b3cd23ffdc3dd429d4cd1905fb8dbcceeff1350020e18b58d2ba70887baa3a9b783ad30d3fbf210331cdd7df8d77defa398cdacdfc2e359c7ba4cae46bb74401deb417f8b912a1aa966aeeba9c39c7dd22479ae2b30719dca2f2206c5eb4b7";
const SECP256K1_SIGNATURE_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
const SECP256K1_PUBLIC_KEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

// TEST 3 test vector from https://tools.ietf.org/html/rfc8032#section-7.1
const ED25519_MESSAGE_HEX: &str = "af82";
const ED25519_SIGNATURE_HEX: &str = "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";
const ED25519_PUBLIC_KEY_HEX: &str =
    "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";

// Signed text "connect all the things" using MyEtherWallet with private key b5b1870957d373ef0eeffecc6e4812c0fd08f554b37b233526acc331bf1544f7
const ETHEREUM_MESSAGE: &str = "connect all the things";
const ETHEREUM_SIGNATURE_HEX: &str = "dada130255a447ecf434a2df9193e6fbba663e4546c35c075cd6eea21d8c7cb1714b9b65a4f7f604ff6aad55fba73f8c36514a512bbbba03709b37069194f8a41b";
const ETHEREUM_SIGNER_ADDRESS: &str = "0x12890D2cce102216644c59daE5baed380d84830c";

// TEST 2 test vector from https://tools.ietf.org/html/rfc8032#section-7.1
const ED25519_MESSAGE2_HEX: &str = "72";
const ED25519_SIGNATURE2_HEX: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";
const ED25519_PUBLIC_KEY2_HEX: &str =
    "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

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
fn cosmos_signature_verify_works() {
    let mut deps = setup();

    let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(SECP256K1_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyCosmosSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn cosmos_signature_verify_fails() {
    let mut deps = setup();

    let mut message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
    // alter hash
    message[0] ^= 0x01;
    let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(SECP256K1_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyCosmosSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn cosmos_signature_verify_errors() {
    let mut deps = setup();

    let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = QueryMsg::VerifyCosmosSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(res.unwrap_err(), "Verification error: Public key error")
}

#[test]
fn ethereum_signature_verify_works() {
    let mut deps = setup();

    let message = ETHEREUM_MESSAGE;
    let signature = hex::decode(ETHEREUM_SIGNATURE_HEX).unwrap();
    let signer_address = ETHEREUM_SIGNER_ADDRESS;

    let verify_msg = QueryMsg::VerifyEthereumText {
        message: message.into(),
        signature: signature.into(),
        signer_address: signer_address.into(),
    };
    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn ethereum_signature_verify_fails_for_corrupted_message() {
    let mut deps = setup();

    let mut message = String::from(ETHEREUM_MESSAGE);
    message.push('!');
    let signature = hex::decode(ETHEREUM_SIGNATURE_HEX).unwrap();
    let signer_address = ETHEREUM_SIGNER_ADDRESS;

    let verify_msg = QueryMsg::VerifyEthereumText {
        message,
        signature: signature.into(),
        signer_address: signer_address.into(),
    };
    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn ethereum_signature_verify_fails_for_corrupted_signature() {
    let mut deps = setup();

    let message = ETHEREUM_MESSAGE;
    let signer_address = ETHEREUM_SIGNER_ADDRESS;

    // Wrong signature
    let mut signature = hex::decode(ETHEREUM_SIGNATURE_HEX).unwrap();
    signature[5] ^= 0x01;
    let verify_msg = QueryMsg::VerifyEthereumText {
        message: message.into(),
        signature: signature.into(),
        signer_address: signer_address.into(),
    };
    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();
    assert_eq!(res, VerifyResponse { verifies: false });

    // Broken signature
    let signature = vec![0x1c; 65];
    let verify_msg = QueryMsg::VerifyEthereumText {
        message: message.into(),
        signature: signature.into(),
        signer_address: signer_address.into(),
    };
    let result = query(&mut deps, mock_env(), verify_msg);
    let msg = result.unwrap_err();
    assert_eq!(msg, "Recover pubkey error: Unknown error: 10");
}

#[test]
fn tendermint_signature_verify_works() {
    let mut deps = setup();

    let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
    let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(ED25519_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyTendermintSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signature_verify_fails() {
    let mut deps = setup();

    let mut message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
    // alter hash
    message[0] ^= 0x01;
    let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(ED25519_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyTendermintSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn tendermint_signature_verify_errors() {
    let mut deps = setup();

    let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
    let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = QueryMsg::VerifyTendermintSignature {
        message: Binary(message),
        signature: Binary(signature),
        public_key: Binary(public_key),
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(res.unwrap_err(), "Verification error: Public key error")
}

#[test]
fn tendermint_signatures_batch_verify_works() {
    let mut deps = setup();

    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_message_multisig_works() {
    let mut deps = setup();

    // One message
    let messages = [ED25519_MESSAGE_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // Multiple signatures
    //FIXME: Use different signatures / pubkeys
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // Multiple pubkeys
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_single_public_key_works() {
    let mut deps = setup();

    // Multiple messages
    //FIXME: Use different messages / signatures
    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // Multiple signatures
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // One pubkey
    let public_keys = [ED25519_PUBLIC_KEY_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_fails() {
    let mut deps = setup();

    let mut messages: Vec<Binary> = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // Alter one of the messages
    messages[1].0[0] ^= 0x01;
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages: (messages),
        signatures: (signatures),
        public_keys: (public_keys),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn tendermint_signatures_batch_verify_errors() {
    let mut deps = setup();

    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();
    // One of the public keys is empty
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ""]
        .iter()
        .map(|m| Binary(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(res.unwrap_err(), "Verification error: Public key error")
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
            verification_schemes: vec![
                "secp256k1".into(),
                "ed25519".into(),
                "ed25519_batch".into()
            ]
        }
    );
}
