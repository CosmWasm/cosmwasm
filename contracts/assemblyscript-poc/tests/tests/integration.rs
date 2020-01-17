use cosmwasm::types::HumanAddr;
use cosmwasm_vm::testing::{mock_instance, query};

use assemblyscript_poc_tests::QueryMsg;

static WASM: &[u8] = include_bytes!("../../contract/build/optimized.wasm");

fn address(index: u8) -> HumanAddr {
    match index {
        0 => HumanAddr("addr0000".to_string()), // contract initializer
        1 => HumanAddr("addr1111".to_string()),
        2 => HumanAddr("addr4321".to_string()),
        3 => HumanAddr("addr5432".to_string()),
        _ => panic!("Unsupported address index"),
    }
}

#[test]
fn can_query_balance_of_existing_address() {
    let mut deps = mock_instance(WASM);

    // TODO: init not yet supported
    // let init_msg = init_msg();
    // let params1 = mock_params_height(&deps.api, &address(0), 450, 550);
    // let res = init(&mut deps, params1, init_msg).unwrap();
    // assert_eq!(0, res.messages.len());

    let query_msg = QueryMsg::Balance {
        address: address(2),
    };
    let query_result = query(&mut deps, query_msg).unwrap();
    assert_eq!(query_result, b"{\"balance\":\"22\"}");
}
