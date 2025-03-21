use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Ibc2PacketReceiveMsg, IbcReceiveResponse, Response, StdError, Empty};

use crate::contract::{ibc2_packet_receive, ibc2_timeout, instantiate, query};
use crate::contract::{QueryMsg, State};

#[test]
fn test_ibc2_timeout() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    // Instantiate the contract
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), Empty {}).unwrap();
    assert_eq!(res, Response::default());

    // Call ibc2_timeout and verify the timeout counter increments
    let msg = Ibc2PacketReceiveMsg::default();
    let res: IbcReceiveResponse = ibc2_timeout(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(res, IbcReceiveResponse::new([1, 2, 3]));

    let query_msg = QueryMsg::QueryTimeoutCounter {};
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let counter: u32 = from_binary(&bin).unwrap();
    assert_eq!(counter, 1);
}

#[test]
fn test_ibc2_timeout_counter_increments() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    // Instantiate the contract
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), Empty {}).unwrap();
    assert_eq!(res, Response::default());

    // Call ibc2_timeout multiple times and verify the timeout counter increments correctly
    let msg = Ibc2PacketReceiveMsg::default();
    for i in 1..=3 {
        let res: IbcReceiveResponse =
            ibc2_timeout(deps.as_mut(), env.clone(), msg.clone()).unwrap();
        assert_eq!(res, IbcReceiveResponse::new([1, 2, 3]));

        let query_msg = QueryMsg::QueryTimeoutCounter {};
        let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let counter: u32 = from_binary(&bin).unwrap();
        assert_eq!(counter, i);
    }
}

#[test]
fn test_query_timeout_counter() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    // Instantiate the contract
    let res = instantiate(deps.as_mut(), env.clone(), info.clone(), Empty {}).unwrap();
    assert_eq!(res, Response::default());

    // Call ibc2_timeout and verify the timeout counter increments
    let msg = Ibc2PacketReceiveMsg::default();
    let res: IbcReceiveResponse = ibc2_timeout(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(res, IbcReceiveResponse::new([1, 2, 3]));

    // Query the timeout counter
    let query_msg = QueryMsg::QueryTimeoutCounter {};
    let bin = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let counter: u32 = from_binary(&bin).unwrap();
    assert_eq!(counter, 1);
}
