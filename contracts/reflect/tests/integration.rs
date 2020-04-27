use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{
    coin, coins, from_binary, Api, ApiError, BankMsg, Binary, Extern, HandleResponse, HandleResult,
    HumanAddr, InitResponse, StakingMsg,
};

use cosmwasm_vm::testing::{handle, init, mock_instance, query};
use cosmwasm_vm::Instance;

use reflect::msg::{CustomMsg, CustomResponse, HandleMsg, InitMsg, OwnerResponse, QueryMsg};
use reflect::testing::CustomQuerier;

/**
This integration test tries to run and call the generated wasm.
It depends on a release build being available already. You can create that with:

cargo wasm

Then running `cargo test` will validate we can properly call into that generated data.

You can easily convert unit tests to integration tests.
1. First copy them over verbatum,
2. Then change
    let mut deps = dependencies(20);
To
    let mut deps = mock_instance(WASM, &[]);
3. If you access raw storage, where ever you see something like:
    deps.storage.get(CONFIG_KEY).expect("no data stored");
 replace it with:
    deps.with_storage(|store| {
        let data = store.get(CONFIG_KEY).expect("no data stored");
        //...
    });
4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)
5. When matching on error codes, you can not use Error types, but rather must use strings:
     match res {
         Err(Error::Unauthorized{..}) => {},
         _ => panic!("Must return unauthorized error"),
     }
     becomes:
     match res {
        ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"),
        _ => panic!("Expected error"),
     }



**/

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/reflect.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse<CustomMsg> = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn reflect() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "creator", &[]);
    let payload = vec![
        BankMsg::Send {
            from_address: deps.api.human_address(&env.contract.address).unwrap(),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into(),
        // make sure we can pass through custom native messages
        CustomMsg::Raw(Binary(b"{\"foo\":123}".to_vec())).into(),
        CustomMsg::Debug("Hi, Dad!".to_string()).into(),
        StakingMsg::Delegate {
            validator: HumanAddr::from("validator"),
            amount: coin(100, "stake"),
        }
        .into(),
    ];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let res: HandleResponse<CustomMsg> = handle(&mut deps, env, msg).unwrap();

    // should return payload
    assert_eq!(payload, res.messages);
}

#[test]
fn reflect_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, env, msg).unwrap();

    // signer is not owner
    let env = mock_env(&deps.api, "someone", &[]);
    let payload = vec![BankMsg::Send {
        from_address: deps.api.human_address(&env.contract.address).unwrap(),
        to_address: HumanAddr::from("friend"),
        amount: coins(1, "token"),
    }
    .into()];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };

    let res: HandleResult<CustomMsg> = handle(&mut deps, env, msg);
    match res {
        Err(ApiError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn transfer() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "creator", &[]);
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };
    let res: HandleResponse<CustomMsg> = handle(&mut deps, env, msg).unwrap();

    // should change state
    assert_eq!(0, res.messages.len());
    let res = query(&mut deps, QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_binary(&res).unwrap();
    assert_eq!("friend", value.owner.as_str());
}

#[test]
fn transfer_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let env = mock_env(&deps.api, "creator", &coins(2, "token"));
    let _res: InitResponse<CustomMsg> = init(&mut deps, env, msg).unwrap();

    let env = mock_env(&deps.api, "random", &[]);
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };

    let res: HandleResult = handle(&mut deps, env, msg);
    match res {
        Err(ApiError::Unauthorized {}) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn dispatch_custom_query() {
    // stub gives us defaults. Consume it and override...
    let custom = mock_dependencies(20, &[]).with_querier(CustomQuerier {});
    // we cannot use mock_instance, so we just copy and modify code from cosmwasm_vm::testing
    let mut deps = Instance::from_code(WASM, custom, 500_000).unwrap();

    // we don't even initialize, just trigger a query
    let res = query(
        &mut deps,
        QueryMsg::ReflectCustom {
            text: "demo one".to_string(),
        },
    )
    .unwrap();
    let value: CustomResponse = from_binary(&res).unwrap();
    assert_eq!("DEMO ONE", value.msg.as_str());
}
