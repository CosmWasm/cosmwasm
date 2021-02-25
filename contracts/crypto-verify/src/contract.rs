use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult, Uint128,
};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::ops::Deref;

use crate::ethereum::{ethereum_address, to_20bytes, verify_transaction};
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
        QueryMsg::VerifyEthereumTransaction {
            from,
            to,
            nonce,
            gas_limit,
            gas_price,
            value,
            data,
            chain_id,
            r,
            s,
            v,
        } => to_binary(&query_verify_ethereum_transaction(
            deps, from, to, nonce, gas_limit, gas_price, value, data, chain_id, r, s, v,
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
        QueryMsg::VerifyTendermintBatch {
            messages,
            signatures,
            public_keys,
        } => to_binary(&query_verify_tendermint_batch(
            deps,
            &messages,
            &signatures,
            &public_keys,
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

pub fn query_verify_ethereum_transaction(
    deps: Deps,
    from: Binary,
    to: Binary,
    nonce: u64,
    gas_limit: Uint128,
    gas_price: Uint128,
    value: Uint128,
    data: Binary,
    chain_id: u64,
    r: Binary,
    s: Binary,
    v: u64,
) -> StdResult<VerifyResponse> {
    let from = to_20bytes(&from)?;
    let to = to_20bytes(&to)?;

    let verifies = verify_transaction(
        deps.api,
        from,
        to,
        nonce,
        gas_limit.into(),
        gas_price.into(),
        value.into(),
        &data,
        chain_id,
        &r,
        &s,
        v,
    )?;
    Ok(VerifyResponse { verifies })
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

pub fn query_verify_tendermint_batch(
    deps: Deps,
    messages: &[Binary],
    signatures: &[Binary],
    public_keys: &[Binary],
) -> StdResult<VerifyResponse> {
    // Deserialization
    let messages: Vec<&[u8]> = messages.iter().map(|b| b.deref()).collect();
    let signatures: Vec<&[u8]> = signatures.iter().map(|b| b.deref()).collect();
    let public_keys: Vec<&[u8]> = public_keys.iter().map(|b| b.deref()).collect();

    // Verification
    let verifies = deps
        .api
        .ed25519_batch_verify(&messages, &signatures, &public_keys)?;
    Ok(VerifyResponse { verifies })
}

pub fn query_list_verifications(deps: Deps) -> StdResult<ListVerificationsResponse> {
    let verification_schemes: Vec<_> = list_verifications(deps)?;
    Ok(ListVerificationsResponse {
        verification_schemes,
    })
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
    use hex_literal::hex;

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
    fn verify_ethereum_transaction_works() {
        let deps = setup();

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
        let from = hex!("0a65766695a712af41b5cfecaad217b1a11cb22a");
        let to = hex!("e137f5264b6b528244e1643a2d570b37660b7f14");
        let gas_limit = Uint128(0x226c8);
        let gas_price = Uint128(0x3b9aca00);
        let value = Uint128(0x53177c);
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
        let raw = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();
        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn tendermint_signatures_batch_verify_works() {
        let deps = setup();

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

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn tendermint_signatures_batch_verify_message_multisig_works() {
        let deps = setup();

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

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn tendermint_signatures_batch_verify_single_public_key_works() {
        let deps = setup();

        // Multiple messages
        //FIXME: Use different messages
        let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE_HEX]
            .iter()
            .map(|m| Binary(hex::decode(m).unwrap()))
            .collect();
        // Multiple signatures
        //FIXME: Use different signatures
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

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: true });
    }

    #[test]
    fn tendermint_signatures_batch_verify_fails() {
        let deps = setup();

        let mut messages: Vec<Binary> = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
            .iter()
            .map(|m| Binary(hex::decode(m).unwrap()))
            .collect();
        // Alter one of the messages
        messages[0].0[0] ^= 0x01;
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

        let raw = query(deps.as_ref(), mock_env(), verify_msg).unwrap();
        let res: VerifyResponse = from_slice(&raw).unwrap();

        assert_eq!(res, VerifyResponse { verifies: false });
    }

    #[test]
    fn tendermint_signatures_batch_verify_errors() {
        let deps = setup();

        let messages = [ED25519_MESSAGE_HEX, ED25519_MESSAGE2_HEX]
            .iter()
            .map(|m| Binary(hex::decode(m).unwrap()))
            .collect();
        let signatures = [ED25519_SIGNATURE_HEX, ED25519_SIGNATURE2_HEX]
            .iter()
            .map(|m| Binary(hex::decode(m).unwrap()))
            .collect();
        // One of the public keys is empty
        let public_keys = ["", ED25519_PUBLIC_KEY_HEX]
            .iter()
            .map(|m| Binary(hex::decode(m).unwrap()))
            .collect();

        let verify_msg = QueryMsg::VerifyTendermintBatch {
            messages,
            signatures,
            public_keys,
        };
        let res = query(deps.as_ref(), mock_env(), verify_msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            StdError::VerificationErr {
                source: VerificationError::PublicKeyErr,
            }
        )
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
                verification_schemes: vec![
                    "secp256k1".into(),
                    "ed25519".into(),
                    "ed25519_batch".into()
                ]
            }
        );
    }
}
