//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.

use cosmwasm_std::{
    coin, coins, from_json, BankMsg, BankQuery, Binary, Coin, ContractResult, Event, QueryRequest,
    Reply, Response, StakingMsg, SubMsg, SubMsgResponse, SubMsgResult, SupplyResponse,
    SystemResult,
};
use cosmwasm_vm::{
    testing::{
        execute, instantiate, mock_env, mock_info, mock_instance, mock_instance_options, query,
        reply, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    },
    Backend, Instance,
};

use reflect::msg::{
    CapitalizedResponse, ChainResponse, CustomMsg, ExecuteMsg, InstantiateMsg, OwnerResponse,
    QueryMsg, SpecialQuery,
};
use reflect::testing::custom_query_execute;

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/reflect.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

/// A drop-in replacement for cosmwasm_vm::testing::mock_dependencies
/// that supports SpecialQuery.
pub fn mock_dependencies_with_custom_querier(
    contract_balance: &[Coin],
) -> Backend<MockApi, MockStorage, MockQuerier<SpecialQuery>> {
    let custom_querier: MockQuerier<SpecialQuery> =
        MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)])
            .with_custom_handler(|query| SystemResult::Ok(custom_query_execute(query)));

    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub fn mock_dependencies_with_custom_querier_and_balances(
    balances: &[(&str, &[Coin])],
) -> Backend<MockApi, MockStorage, MockQuerier<SpecialQuery>> {
    let custom_querier: MockQuerier<SpecialQuery> = MockQuerier::new(balances)
        .with_custom_handler(|query| SystemResult::Ok(custom_query_execute(query)));

    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_json(res).unwrap();
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn reflect() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    let payload = vec![
        BankMsg::Send {
            to_address: String::from("friend"),
            amount: coins(1, "token"),
        }
        .into(),
        // make sure we can pass through custom native messages
        CustomMsg::Raw(Binary::new(b"{\"foo\":123}".to_vec())).into(),
        CustomMsg::Debug("Hi, Dad!".to_string()).into(),
        StakingMsg::Delegate {
            validator: String::from("validator"),
            amount: coin(100, "ustake"),
        }
        .into(),
    ];
    let msg = ExecuteMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let info = mock_info("creator", &[]);
    let res: Response = execute(&mut deps, mock_env(), info, msg).unwrap();

    // should return payload. We're comparing the JSON representation because the underlying
    // CustomMsg type is not the same.
    let payload =
        cosmwasm_std::to_json_string(&payload.into_iter().map(SubMsg::new).collect::<Vec<_>>())
            .unwrap();
    let messages = cosmwasm_std::to_json_string(&res.messages).unwrap();
    assert_eq!(payload, messages);
}

#[test]
fn reflect_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    // signer is not owner
    let payload = vec![BankMsg::Send {
        to_address: String::from("friend"),
        amount: coins(1, "token"),
    }
    .into()];
    let msg = ExecuteMsg::ReflectMsg { msgs: payload };

    let info = mock_info("someone", &[]);
    let res: ContractResult<Response> = execute(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert!(msg.contains("Permission denied: the sender is not the current owner"));
}

#[test]
fn transfer() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    let info = mock_info("creator", &[]);
    let new_owner = deps.api().addr_make("friend");
    let msg = ExecuteMsg::ChangeOwner {
        owner: new_owner.to_string(),
    };
    let res: Response = execute(&mut deps, mock_env(), info, msg).unwrap();

    // should change state
    assert_eq!(0, res.messages.len());
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_json(res).unwrap();
    assert_eq!(value.owner, new_owner.as_str());
}

#[test]
fn transfer_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    let info = mock_info("random", &[]);
    let new_owner = String::from("friend");
    let msg = ExecuteMsg::ChangeOwner { owner: new_owner };

    let res: ContractResult<Response> = execute(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert!(msg.contains("Permission denied: the sender is not the current owner"));
}

#[test]
fn supply_query() {
    // stub gives us defaults. Consume it and override...
    let custom = mock_dependencies_with_custom_querier_and_balances(&[
        ("ryan_reynolds", &[coin(5, "ATOM"), coin(10, "OSMO")]),
        ("huge_ackman", &[coin(15, "OSMO"), coin(5, "BTC")]),
    ]);
    // we cannot use mock_instance, so we just copy and modify code from cosmwasm_vm::testing
    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, custom, instance_options, memory_limit).unwrap();

    // we don't even initialize, just trigger a query
    let res = query(
        &mut deps,
        mock_env(),
        QueryMsg::Chain {
            request: QueryRequest::Bank(BankQuery::Supply {
                denom: "OSMO".to_string(),
            }),
        },
    )
    .unwrap();

    let res: ChainResponse = from_json(res).unwrap();
    let res: SupplyResponse = from_json(res.data).unwrap();
    assert_eq!(res.amount, coin(25, "OSMO"));
}

#[test]
fn dispatch_custom_query() {
    // stub gives us defaults. Consume it and override...
    let custom = mock_dependencies_with_custom_querier(&[]);
    // we cannot use mock_instance, so we just copy and modify code from cosmwasm_vm::testing
    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, custom, instance_options, memory_limit).unwrap();

    // we don't even initialize, just trigger a query
    let res = query(
        &mut deps,
        mock_env(),
        QueryMsg::Capitalized {
            text: "demo one".to_string(),
        },
    )
    .unwrap();
    let value: CapitalizedResponse = from_json(res).unwrap();
    assert_eq!(value.text, "DEMO ONE");
}

#[test]
fn reflect_subcall() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    let id = 123u64;
    let payload = SubMsg::reply_always(
        BankMsg::Send {
            to_address: String::from("friend"),
            amount: coins(1, "token"),
        },
        id,
    );

    let msg = ExecuteMsg::ReflectSubMsg {
        msgs: vec![payload.clone()],
    };
    let info = mock_info("creator", &[]);
    let mut res: Response = execute(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
    let msg = res.messages.pop().expect("must have a message");
    assert_eq!(payload, msg);
}

// this mocks out what happens after reflect_subcall
#[test]
fn reply_and_query() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InstantiateMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: Response = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    let id = 123u64;
    let payload = Binary::from(b"my dear");
    let data = Binary::from(b"foobar");
    let events = vec![Event::new("message").add_attribute("signer", "caller-addr")];
    let gas_used = 1234567u64;
    #[allow(deprecated)]
    let result = SubMsgResult::Ok(SubMsgResponse {
        events: events.clone(),
        data: Some(data.clone()),
        msg_responses: vec![],
    });
    let the_reply = Reply {
        id,
        payload,
        gas_used,
        result,
    };
    let res: Response = reply(&mut deps, mock_env(), the_reply).unwrap();
    assert_eq!(0, res.messages.len());

    // query for a non-existent id
    let qres = query(&mut deps, mock_env(), QueryMsg::SubMsgResult { id: 65432 });
    assert!(qres.is_err());

    // query for the real id
    let raw = query(&mut deps, mock_env(), QueryMsg::SubMsgResult { id }).unwrap();
    let qres: Reply = from_json(raw).unwrap();
    assert_eq!(qres.id, id);
    let result = qres.result.unwrap();
    #[allow(deprecated)]
    {
        assert_eq!(result.data, Some(data));
    }
    assert_eq!(result.events, events);
}
