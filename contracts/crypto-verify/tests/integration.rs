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
//! 4. Anywhere you see init/execute(deps.as_mut(), ...) you must replace it with init/execute(&mut deps, ...)
//! 5. Anywhere you see query(deps.as_ref(), ...) you must replace it with query(&mut deps, ...)
//! (Use cosmwasm_vm::testing::{init, execute, query}, instead of the contract variants).

use cosmwasm_std::{Binary, Response, Uint128};
use cosmwasm_vm::testing::{
    instantiate, mock_env, mock_info, mock_instance_with_gas_limit, query, MockApi, MockQuerier,
    MockStorage,
};
use cosmwasm_vm::{from_slice, Instance};
use hex_literal::hex;
use sha2::{Digest, Sha256};

use crypto_verify::msg::{InstantiateMsg, ListVerificationsResponse, QueryMsg, VerifyResponse};

// Output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/crypto_verify.wasm");

const CREATOR: &str = "creator";

const SECP256K1_MESSAGE_HEX: &str = "5c868fedb8026979ebd26f1ba07c27eedf4ff6d10443505a96ecaf21ba8c4f0937b3cd23ffdc3dd429d4cd1905fb8dbcceeff1350020e18b58d2ba70887baa3a9b783ad30d3fbf210331cdd7df8d77defa398cdacdfc2e359c7ba4cae46bb74401deb417f8b912a1aa966aeeba9c39c7dd22479ae2b30719dca2f2206c5eb4b7";
const SECP256K1_SIGNATURE_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
const SECP256K1_PUBLIC_KEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

const SECP256R1_MESSAGE_HEX: &str =
    "4d55c99ef6bd54621662c3d110c3cb627c03d6311393b264ab97b90a4b15214a5593ba2510a53d63fb34be251facb697c973e11b665cb7920f1684b0031b4dd370cb927ca7168b0bf8ad285e05e9e31e34bc24024739fdc10b78586f29eff94412034e3b606ed850ec2c1900e8e68151fc4aee5adebb066eb6da4eaa5681378e";
const SECP256R1_SIGNATURE_HEX: &str = "1cc628533d0004b2b20e7f4baad0b8bb5e0673db159bbccf92491aef61fc9620880e0bbf82a8cf818ed46ba03cf0fc6c898e36fca36cc7fdb1d2db7503634430";
const SECP256R1_PUBLIC_KEY_HEX: &str = "04b8188bd68701fc396dab53125d4d28ea33a91daf6d21485f4770f6ea8c565dde423f058810f277f8fe076f6db56e9285a1bf2c2a1dae145095edd9c04970bc4a";

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

// Vectors sourced from <https://github.com/daimo-eth/p256-verifier/blob/master/test/WebAuthn.t.sol>
const WEBAUTHN_PUBLIC_KEY_X: &[u8] =
    &hex!("80d9326e49eb6314d03f58830369ea5bafbc4e2709b30bff1f4379586ca869d9");
const WEBAUTHN_PUBLIC_KEY_Y: &[u8] =
    &hex!("806ed746d8ac6c2779a472d8c1ed4c200b07978d9d8d8d862be8b7d4b7fb6350");
const WEBAUTHN_CLIENT_DATA_JSON: &str = r#"{"type":"webauthn.get","challenge":"dGVzdA","origin":"https://funny-froyo-3f9b75.netlify.app"}"#;
const WEBAUTHN_CHALLENGE: &[u8] = &hex!("74657374");
const WEBAUTHN_AUTHENTICATOR_DATA: &[u8] =
    &hex!("e0b592a7dd54eedeec65206e031fc196b8e5915f9b389735860c83854f65dc0e1d00000000");
const WEBAUTHN_SIGNATURE_R: &[u8] =
    &hex!("32e005a53ae49a96ac88c715243638dd5c985fbd463c727d8eefd05bee4e2570");
const WEBAUTHN_SIGNATURE_S: &[u8] =
    &hex!("7a4fef4d0b11187f95f69eefbb428df8ac799bbd9305066b1e9c9fe9a5bcf8c4");

// See https://github.com/drand/kyber-bls12381/issues/22 and
// https://github.com/drand/drand/pull/1249
const DOMAIN_HASH_TO_G1: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";
const DOMAIN_HASH_TO_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

/// Public key League of Entropy Mainnet (curl -sS https://drand.cloudflare.com/info)
const PK_LEO_MAINNET: [u8; 48] = hex!("868f005eb8e6e4ca0a47c8a77ceaa5309a47978a7c71bc5cce96366b5d7a569937c529eeda66c7293784a9402801af31");

// Tests from quicknet (https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/info)
const PK_QUICKNET: [u8; 96] = hex!("83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a");

fn build_drand_message(round: u64, previous_signature: &[u8]) -> Vec<u8> {
    Sha256::new()
        .chain_update(previous_signature)
        .chain_update(round.to_be_bytes())
        .finalize()
        .to_vec()
}

const DESERIALIZATION_LIMIT: usize = 20_000;

fn setup() -> Instance<MockApi, MockStorage, MockQuerier> {
    let mut deps = mock_instance_with_gas_limit(WASM, 10_000_000_000);
    let msg = InstantiateMsg {};
    let info = mock_info(CREATOR, &[]);
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

#[test]
fn instantiate_works() {
    setup();
}

#[test]
fn bls12_381_verifies_g1() {
    let mut deps = setup();

    let previous_signature = hex::decode("a609e19a03c2fcc559e8dae14900aaefe517cb55c840f6e69bc8e4f66c8d18e8a609685d9917efbfb0c37f058c2de88f13d297c7e19e0ab24813079efe57a182554ff054c7638153f9b26a60e7111f71a0ff63d9571704905d3ca6df0b031747").unwrap();
    let signature = hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42").unwrap();
    let round: u64 = 72785;

    let msg = build_drand_message(round, &previous_signature);

    let verify_msg = QueryMsg::VerifyBls12PairingEqualityG1 {
        signature: signature.into(),
        pubkey: PK_LEO_MAINNET.into(),
        msg: msg.into(),
        dst: DOMAIN_HASH_TO_G2.into(),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn bls12_381_verifies_g2() {
    let mut deps = setup();

    let signature = hex::decode("b75c69d0b72a5d906e854e808ba7e2accb1542ac355ae486d591aa9d43765482e26cd02df835d3546d23c4b13e0dfc92").unwrap();
    let round: u64 = 123;

    let msg = build_drand_message(round, b"");

    let verify_msg = QueryMsg::VerifyBls12PairingEqualityG2 {
        signature: signature.into(),
        pubkey: PK_QUICKNET.into(),
        msg: msg.into(),
        dst: DOMAIN_HASH_TO_G1.into(),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn bls12_381_errors() {
    let mut deps = setup();

    let mut previous_signature = hex::decode("a609e19a03c2fcc559e8dae14900aaefe517cb55c840f6e69bc8e4f66c8d18e8a609685d9917efbfb0c37f058c2de88f13d297c7e19e0ab24813079efe57a182554ff054c7638153f9b26a60e7111f71a0ff63d9571704905d3ca6df0b031747").unwrap();
    let signature = hex::decode("82f5d3d2de4db19d40a6980e8aa37842a0e55d1df06bd68bddc8d60002e8e959eb9cfa368b3c1b77d18f02a54fe047b80f0989315f83b12a74fd8679c4f12aae86eaf6ab5690b34f1fddd50ee3cc6f6cdf59e95526d5a5d82aaa84fa6f181e42").unwrap();
    let round: u64 = 72785;

    previous_signature[0] ^= 0x3;

    let msg = build_drand_message(round, &previous_signature);

    let verify_msg = QueryMsg::VerifyBls12PairingEqualityG1 {
        signature: signature.into(),
        pubkey: PK_LEO_MAINNET.into(),
        msg: msg.into(),
        dst: DOMAIN_HASH_TO_G2.into(),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn cosmos_signature_verify_works() {
    let mut deps = setup();

    let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(SECP256K1_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyCosmosSignature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

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
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn cosmos_signature_verify_errors() {
    let mut deps = setup();

    let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = QueryMsg::VerifyCosmosSignature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(
        res.unwrap_err(),
        "Verification error: Invalid public key format"
    )
}

#[test]
fn secp256r1_signature_verify_works() {
    let mut deps = setup();

    let message = hex::decode(SECP256R1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256R1_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(SECP256R1_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifySecp256R1Signature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn secp256r1_signature_verify_fails() {
    let mut deps = setup();

    let mut message = hex::decode(SECP256R1_MESSAGE_HEX).unwrap();
    // alter hash
    message[0] ^= 0x01;
    let signature = hex::decode(SECP256R1_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(SECP256R1_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifySecp256R1Signature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn secp256r1_signature_verify_errors() {
    let mut deps = setup();

    let message = hex::decode(SECP256R1_MESSAGE_HEX).unwrap();
    let signature = hex::decode(SECP256R1_SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = QueryMsg::VerifySecp256R1Signature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(
        res.unwrap_err(),
        "Verification error: Invalid public key format"
    )
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
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

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
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

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
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
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
fn verify_ethereum_transaction_works() {
    let mut deps = setup();

    // curl -sS -X POST --data '{"jsonrpc":"2.0","method":"eth_getTransactionByHash","params":["0x3b87faa3410f33284124a6898fac1001673f0f7c3682d18f55bdff0031cce9ce"],"id":1}' -H "Content-type: application/json" https://rinkeby-light.eth.linkpool.io | jq .result
    // {
    //   "blockHash": "0x05ebd1bd99956537f49cfa1104682b3b3f9ff9249fa41a09931ce93368606c21",
    //   "blockNumber": "0x37ef3e",
    //   "from": "0x0a65766695a712af41b5cfecaad217b1a11cb22a",
    //   "gas": "0x226c8",
    //   "gasPrice": "0x3b9aca00",
    //   "hash": "0x3b87faa3410f33284124a6898fac1001673f0f7c3682d18f55bdff0031cce9ce",
    //   "input": "0x536561726368207478207465737420302e36353930383639313733393634333335",
    //   "nonce": "0xe1",
    //   "to": "0xe137f5264b6b528244e1643a2d570b37660b7f14",
    //   "transactionIndex": "0xb",
    //   "value": "0x53177c",
    //   "v": "0x2b",
    //   "r": "0xb9299dab50b3cddcaecd64b29bfbd5cd30fac1a1adea1b359a13c4e5171492a6",
    //   "s": "0x573059c66d894684488f92e7ce1f91b158ca57b0235485625b576a3b98c480ac"
    // }
    let nonce = 0xe1;
    let chain_id = 4; // Rinkeby, see https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md#list-of-chain-ids
    let from = "0x0a65766695a712af41b5cfecaad217b1a11cb22a";
    let to = "0xe137f5264b6b528244e1643a2d570b37660b7f14";
    let gas_limit = Uint128::new(0x226c8);
    let gas_price = Uint128::new(0x3b9aca00);
    let value = Uint128::new(0x53177c);
    let data = hex!("536561726368207478207465737420302e36353930383639313733393634333335");
    let r = hex!("b9299dab50b3cddcaecd64b29bfbd5cd30fac1a1adea1b359a13c4e5171492a6");
    let s = hex!("573059c66d894684488f92e7ce1f91b158ca57b0235485625b576a3b98c480ac");
    let v = 0x2b;

    let msg = QueryMsg::VerifyEthereumTransaction {
        from: from.into(),
        to: to.into(),
        nonce,
        gas_limit,
        gas_price,
        value,
        data: data.into(),
        chain_id,
        r: r.into(),
        s: s.into(),
        v,
    };
    let raw = query(&mut deps, mock_env(), msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signature_verify_works() {
    let mut deps = setup();

    let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
    let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
    let public_key = hex::decode(ED25519_PUBLIC_KEY_HEX).unwrap();

    let verify_msg = QueryMsg::VerifyTendermintSignature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

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
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn tendermint_signature_verify_errors() {
    let mut deps = setup();

    let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
    let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
    let public_key = vec![];

    let verify_msg = QueryMsg::VerifyTendermintSignature {
        message: Binary::new(message),
        signature: Binary::new(signature),
        public_key: Binary::new(public_key),
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(
        res.unwrap_err(),
        "Verification error: Invalid public key format"
    )
}

#[test]
fn tendermint_signatures_batch_verify_works() {
    let mut deps = setup();

    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_message_multisig_works() {
    let mut deps = setup();

    // One message
    let messages = [ED25519_MESSAGE_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // Multiple signatures
    //FIXME: Use different signatures / pubkeys
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // Multiple pubkeys
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_single_public_key_works() {
    let mut deps = setup();

    // Multiple messages
    //FIXME: Use different messages / signatures
    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // Multiple signatures
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // One pubkey
    let public_keys = [ED25519_PUBLIC_KEY_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: true });
}

#[test]
fn tendermint_signatures_batch_verify_fails() {
    let mut deps = setup();

    let mut messages: Vec<Binary> = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // Alter one of the messages
    let mut b: Vec<_> = messages.swap_remove(0).into();
    b[0] ^= 0x01;
    messages.push(Binary::new(b));
    messages.swap(0, 1); // swap them again to old order
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ED25519_PUBLIC_KEY2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages: (messages),
        signatures: (signatures),
        public_keys: (public_keys),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(res, VerifyResponse { verifies: false });
}

#[test]
fn tendermint_signatures_batch_verify_errors() {
    let mut deps = setup();

    let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();
    // One of the public keys is empty
    let public_keys = [ED25519_PUBLIC_KEY_HEX, ""]
        .iter()
        .map(|m| Binary::new(hex::decode(m).unwrap()))
        .collect();

    let verify_msg = QueryMsg::VerifyTendermintBatch {
        messages,
        signatures,
        public_keys,
    };
    let res = query(&mut deps, mock_env(), verify_msg);
    assert_eq!(
        res.unwrap_err(),
        "Verification error: Invalid public key format"
    )
}

#[test]
fn webauthn_verify_works() {
    let mut deps = setup();
    let verify_msg = QueryMsg::VerifyWebauthn {
        authenticator_data: WEBAUTHN_AUTHENTICATOR_DATA.into(),
        client_data_json: WEBAUTHN_CLIENT_DATA_JSON.into(),
        challenge: WEBAUTHN_CHALLENGE.into(),
        x: WEBAUTHN_PUBLIC_KEY_X.into(),
        y: WEBAUTHN_PUBLIC_KEY_Y.into(),
        r: WEBAUTHN_SIGNATURE_R.into(),
        s: WEBAUTHN_SIGNATURE_S.into(),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert!(res.verifies);
}

#[test]
fn webauthn_verify_errors() {
    let mut deps = setup();

    let mut r = WEBAUTHN_SIGNATURE_R.to_vec();
    r[0] ^= 3;

    let verify_msg = QueryMsg::VerifyWebauthn {
        authenticator_data: WEBAUTHN_AUTHENTICATOR_DATA.into(),
        client_data_json: WEBAUTHN_CLIENT_DATA_JSON.into(),
        challenge: WEBAUTHN_CHALLENGE.into(),
        x: WEBAUTHN_PUBLIC_KEY_X.into(),
        y: WEBAUTHN_PUBLIC_KEY_Y.into(),
        r: r.into(),
        s: WEBAUTHN_SIGNATURE_S.into(),
    };

    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert!(!res.verifies);

    let mut client_data_json = WEBAUTHN_CLIENT_DATA_JSON.to_string();
    client_data_json.push_str("tampering with hashes is fun");
    let verify_msg = QueryMsg::VerifyWebauthn {
        authenticator_data: WEBAUTHN_AUTHENTICATOR_DATA.into(),
        client_data_json,
        challenge: WEBAUTHN_CHALLENGE.into(),
        x: WEBAUTHN_PUBLIC_KEY_X.into(),
        y: WEBAUTHN_PUBLIC_KEY_Y.into(),
        r: WEBAUTHN_SIGNATURE_R.into(),
        s: WEBAUTHN_SIGNATURE_S.into(),
    };

    let mut deps = setup();
    let raw = query(&mut deps, mock_env(), verify_msg).unwrap();
    let res: VerifyResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();
    assert!(!res.verifies);
}

#[test]
fn query_works() {
    let mut deps = setup();

    let query_msg = QueryMsg::ListVerificationSchemes {};

    let raw = query(&mut deps, mock_env(), query_msg).unwrap();
    let res: ListVerificationsResponse = from_slice(&raw, DESERIALIZATION_LIMIT).unwrap();

    assert_eq!(
        res,
        ListVerificationsResponse {
            verification_schemes: vec![
                "secp256k1".into(),
                "secp256r1".into(),
                "ed25519".into(),
                "ed25519_batch".into()
            ]
        }
    );
}
