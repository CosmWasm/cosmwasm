use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
};
use sha2::{Digest, Sha256};
use sha3::Keccak256;

use crate::msg::{
    list_verifications, HandleMsg, InitMsg, ListVerificationsResponse, QueryMsg, VerifyResponse,
};

pub const VERSION: &str = "crypto-verify-v2";

#[entry_point]
pub fn init(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InitMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn handle(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: HandleMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::VerifyCosmosSignature {
            message,
            signature,
            public_key,
        } => to_binary(&query_verify_cosmos(
            deps,
            &message.0,
            &signature.0,
            &public_key.0,
        )?),
        QueryMsg::VerifyEthereumText {
            message,
            signature,
            signer_address,
        } => to_binary(&query_verify_ethereum_text(
            deps,
            &message,
            &signature,
            &signer_address,
        )?),
        QueryMsg::VerifyTendermintSignature {
            message,
            signature,
            public_key,
        } => to_binary(&query_verify_tendermint(
            deps,
            &message.0,
            &signature.0,
            &public_key.0,
        )?),
        QueryMsg::ListVerificationSchemes {} => to_binary(&query_list_verifications(deps)?),
    }
}

pub fn query_verify_cosmos(
    deps: Deps,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<VerifyResponse> {
    // Hashing
    let hash = Sha256::digest(message);

    // Verification
    let result = deps
        .api
        .secp256k1_verify(hash.as_ref(), signature, public_key);
    match result {
        Ok(verifies) => Ok(VerifyResponse { verifies }),
        Err(err) => Err(err.into()),
    }
}

pub fn query_verify_ethereum_text(
    deps: Deps,
    message: &str,
    signature: &[u8],
    signer_address: &str,
) -> StdResult<VerifyResponse> {
    // Hashing
    let mut hasher = Keccak256::new();
    hasher.update(format!("\x19Ethereum Signed Message:\n{}", message.len()));
    hasher.update(message);
    let hash = hasher.finalize();

    // Decompose signature
    let (v, rs) = match signature.split_last() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Signature must not be empty")),
    };
    let recovery = get_recovery_param(*v)?;

    // Verification
    let calculated_pubkey = deps.api.secp256k1_recover_pubkey(&hash, rs, recovery)?;
    let calculated_address = ethereum_address(&calculated_pubkey)?;
    if signer_address.to_ascii_lowercase() != calculated_address {
        return Ok(VerifyResponse { verifies: false });
    }
    let result = deps.api.secp256k1_verify(&hash, rs, &calculated_pubkey);
    match result {
        Ok(verifies) => Ok(VerifyResponse { verifies }),
        Err(err) => Err(err.into()),
    }
}

pub fn query_verify_tendermint(
    deps: Deps,
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<VerifyResponse> {
    // Verification
    let verifies = deps.api.ed25519_verify(message, signature, public_key)?;
    Ok(VerifyResponse { verifies })
}

pub fn query_list_verifications(deps: Deps) -> StdResult<ListVerificationsResponse> {
    let verification_schemes: Vec<_> = list_verifications(deps)?;
    Ok(ListVerificationsResponse {
        verification_schemes,
    })
}

fn ethereum_address(pubkey: &[u8]) -> StdResult<String> {
    let (tag, data) = match pubkey.split_first() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Public key must not be empty")),
    };
    if *tag != 0x04 {
        return Err(StdError::generic_err("Public key start with 0x04"));
    }
    if data.len() != 64 {
        return Err(StdError::generic_err("Public key must be 65 bytes long"));
    }

    let hash = Keccak256::digest(data);
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    out.push_str(&hex::encode(&hash[hash.len() - 20..]));
    Ok(out)
}

fn get_recovery_param(v: u8) -> StdResult<u8> {
    // See https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md
    // for how `v` is composed.
    match v {
        27 => Ok(0),
        28 => Ok(1),
        _ => Err(StdError::generic_err("Values of v other than 27 and 28 not supported. Replay protection (EIP-155) cannot be used here."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_slice, Binary, OwnedDeps, RecoverPubkeyError, StdError, VerificationError,
    };

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

    fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(&[]);
        let msg = InitMsg {};
        let info = mock_info(CREATOR, &[]);
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        deps
    }

    #[test]
    fn init_works() {
        setup();
    }

    #[test]
    fn cosmos_signature_verify_works() {
        let deps = setup();

        let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBLIC_KEY_HEX).unwrap();

        let verify_msg = QueryMsg::VerifyCosmosSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn cosmos_signature_verify_fails() {
        let deps = setup();

        let mut message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
        // alter message
        message[0] ^= 0x01;
        let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(SECP256K1_PUBLIC_KEY_HEX).unwrap();

        let verify_msg = QueryMsg::VerifyCosmosSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: false });
    }

    #[test]
    fn cosmos_signature_verify_errors() {
        let deps = setup();

        let message = hex::decode(SECP256K1_MESSAGE_HEX).unwrap();
        let signature = hex::decode(SECP256K1_SIGNATURE_HEX).unwrap();
        let public_key = vec![];

        let verify_msg = QueryMsg::VerifyCosmosSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };

        let res = query(deps.as_ref(), mock_env(), verify_msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            StdError::VerificationErr {
                source: VerificationError::PublicKeyErr
            }
        )
    }

    #[test]
    fn ethereum_signature_verify_works() {
        let deps = setup();

        let message = ETHEREUM_MESSAGE;
        let signature = hex::decode(ETHEREUM_SIGNATURE_HEX).unwrap();
        let signer_address = ETHEREUM_SIGNER_ADDRESS;

        let verify_msg = QueryMsg::VerifyEthereumText {
            message: message.into(),
            signature: signature.into(),
            signer_address: signer_address.into(),
        };
        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn ethereum_signature_verify_fails_for_corrupted_message() {
        let deps = setup();

        let mut message = String::from(ETHEREUM_MESSAGE);
        message.push('!');
        let signature = hex::decode(ETHEREUM_SIGNATURE_HEX).unwrap();
        let signer_address = ETHEREUM_SIGNER_ADDRESS;

        let verify_msg = QueryMsg::VerifyEthereumText {
            message,
            signature: signature.into(),
            signer_address: signer_address.into(),
        };
        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: false });
    }

    #[test]
    fn ethereum_signature_verify_fails_for_corrupted_signature() {
        let deps = setup();

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
        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();
        assert_eq!(res, VerifyResponse { verifies: false });

        // Broken signature
        let signature = vec![0x1c; 65];
        let verify_msg = QueryMsg::VerifyEthereumText {
            message: message.into(),
            signature: signature.into(),
            signer_address: signer_address.into(),
        };
        let result = query(deps.as_ref(), mock_env(), verify_msg);
        match result.unwrap_err() {
            StdError::RecoverPubkeyErr {
                source: RecoverPubkeyError::UnknownErr { .. },
                ..
            } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn tendermint_signature_verify_works() {
        let deps = setup();

        let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
        let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBLIC_KEY_HEX).unwrap();

        let verify_msg = QueryMsg::VerifyTendermintSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn tendermint_signature_verify_fails() {
        let deps = setup();

        let mut message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
        // alter message
        message[0] ^= 0x01;
        let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
        let public_key = hex::decode(ED25519_PUBLIC_KEY_HEX).unwrap();

        let verify_msg = QueryMsg::VerifyTendermintSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: false });
    }

    #[test]
    fn tendermint_signature_verify_errors() {
        let deps = setup();

        let message = hex::decode(ED25519_MESSAGE_HEX).unwrap();
        let signature = hex::decode(ED25519_SIGNATURE_HEX).unwrap();
        let public_key = vec![];

        let verify_msg = QueryMsg::VerifyTendermintSignature {
            message: Binary(message),
            signature: Binary(signature),
            public_key: Binary(public_key),
        };
        let res = query(deps.as_ref(), mock_env(), verify_msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            StdError::VerificationErr {
                source: VerificationError::PublicKeyErr
            }
        )
    }

    #[test]
    fn list_signatures_works() {
        let deps = setup();

        let query_msg = QueryMsg::ListVerificationSchemes {};

        let raw = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let res: ListVerificationsResponse = from_slice(&raw).unwrap();

        assert_eq!(
            res,
            ListVerificationsResponse {
                verification_schemes: vec!["secp256k1".into(), "ed25519".into()]
            }
        );
    }
}
