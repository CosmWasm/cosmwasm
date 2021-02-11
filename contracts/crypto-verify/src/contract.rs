use cosmwasm_std::{
    attr, entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult,
};

use crate::msg::list_verifications;
use crate::msg::{HandleMsg, InitMsg, ListVerificationsResponse, QueryMsg};

pub const VERSION: &str = "crypto-verify-v1";

#[entry_point]
pub fn init(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InitMsg) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn handle(deps: DepsMut, _env: Env, _info: MessageInfo, msg: HandleMsg) -> StdResult<Response> {
    match msg {
        HandleMsg::VerifySignature {
            message_hash,
            signature,
            public_key,
        } => handle_verify(deps, &message_hash.0, &signature.0, &public_key.0),
    }
}

pub fn handle_verify(
    deps: DepsMut,
    hash: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<Response> {
    // Verification
    let verify = deps.api.secp256k1_verify(hash, signature, public_key);

    Ok(Response {
        messages: vec![],
        attributes: vec![attr("action", "handle_verify")],
        data: Some(Binary(vec![verify.into()])),
    })
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::ListVerificationSchemes {} => to_binary(&query_list_verifications(deps)?),
    }
}

pub fn query_list_verifications(deps: Deps) -> StdResult<ListVerificationsResponse> {
    let verification_schemes: Vec<_> = list_verifications(deps)?;
    Ok(ListVerificationsResponse {
        verification_schemes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{from_slice, OwnedDeps};

    const CREATOR: &str = "creator";
    const SENDER: &str = "sender";

    const HASH_HEX: &str = "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const SIGNATURE_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const PUBLIC_KEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

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
        let res = handle(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[]),
            verify_msg,
        )
        .unwrap();
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
        let res = handle(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[]),
            verify_msg,
        )
        .unwrap();
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
        handle(
            deps.as_mut(),
            mock_env(),
            mock_info(SENDER, &[]),
            verify_msg,
        )
        .unwrap();
    }

    #[test]
    fn query_works() {
        let deps = setup();

        let query_msg = QueryMsg::ListVerificationSchemes {};

        let raw = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let res: ListVerificationsResponse = from_slice(&raw).unwrap();

        assert_eq!(
            res,
            ListVerificationsResponse {
                verification_schemes: vec!["secp256k1".into()]
            }
        );
    }
}
