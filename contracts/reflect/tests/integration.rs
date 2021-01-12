//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
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
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{
    coin, coins, from_binary, BankMsg, Binary, Coin, ContractResult, HandleResponse, HumanAddr,
    InitResponse, StakingMsg, SystemResult,
};
use cosmwasm_vm::{
    testing::{
        handle, init, mock_env, mock_info, mock_instance, mock_instance_options, query, MockApi,
        MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    },
    Backend, Instance,
};

use reflect::msg::{
    CapitalizedResponse, CustomMsg, HandleMsg, InitMsg, OwnerResponse, QueryMsg, SpecialQuery,
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
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let custom_querier: MockQuerier<SpecialQuery> =
        MockQuerier::new(&[(&contract_addr, contract_balance)])
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

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse<CustomMsg> = init(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn reflect() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, mock_env(), info, msg).unwrap();

    let payload = vec![
        BankMsg::Send {
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into(),
        // make sure we can pass through custom native messages
        CustomMsg::Raw(Binary(b"{\"foo\":123}".to_vec())).into(),
        CustomMsg::Debug("Hi, Dad!".to_string()).into(),
        StakingMsg::Delegate {
            validator: HumanAddr::from("validator"),
            amount: coin(100, "ustake"),
        }
        .into(),
    ];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let info = mock_info("creator", &[]);
    let res: HandleResponse<CustomMsg> = handle(&mut deps, mock_env(), info, msg).unwrap();

    // should return payload
    assert_eq!(payload, res.messages);
}

#[test]
fn reflect_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, mock_env(), info, msg).unwrap();

    // signer is not owner
    let payload = vec![BankMsg::Send {
        to_address: HumanAddr::from("friend"),
        amount: coins(1, "token"),
    }
    .into()];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };

    let info = mock_info("someone", &[]);
    let res: ContractResult<HandleResponse<CustomMsg>> = handle(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert!(msg.contains("Permission denied: the sender is not the current owner"));
}

#[test]
fn transfer() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, mock_env(), info, msg).unwrap();

    let info = mock_info("creator", &[]);
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };
    let res: HandleResponse<CustomMsg> = handle(&mut deps, mock_env(), info, msg).unwrap();

    // should change state
    assert_eq!(0, res.messages.len());
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!("friend", value.owner.as_str());
}

#[test]
fn transfer_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, mock_env(), info, msg).unwrap();

    let info = mock_info("random", &[]);
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };

    let res: ContractResult<HandleResponse> = handle(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert!(msg.contains("Permission denied: the sender is not the current owner"));
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
    let value: CapitalizedResponse = from_binary(&res).unwrap();
    assert_eq!(value.text, "DEMO ONE");
}
