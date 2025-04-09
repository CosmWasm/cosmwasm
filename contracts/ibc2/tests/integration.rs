use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_binary, Empty, Ibc2PacketReceiveMsg, IbcReceiveResponse, Response, StdError,
};

use crate::contract::{ibc2_packet_receive, ibc2_timeout, instantiate, query};
use crate::contract::{QueryMsg, State};

#[test]
fn test_ibc2_timeout() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    // Instantiate the contract
    let msg = Empty {};
    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res, Response::new());

    // Call ibc2_timeout multiple times and verify the timeout counter increments correctly
    let msg = Ibc2PacketReceiveMsg::default();
    for i in 1..=3 {
        let res: IbcReceiveResponse =
            ibc2_timeout(deps.as_mut(), env.clone(), msg.clone()).unwrap();
        assert_eq!(res, IbcReceiveResponse::new([1, 2, 3]));

        let query_msg = QueryMsg::QueryTimeoutCounter {};
        let query_res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let timeout_counter: u32 = from_binary(&query_res).unwrap();
        assert_eq!(timeout_counter, i);
    }
}
