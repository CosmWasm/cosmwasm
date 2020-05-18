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
    coin, coins, from_binary, BankMsg, Binary, HandleResponse, HandleResult, HumanAddr,
    InitResponse, StakingMsg, StdError,
};
use cosmwasm_vm::{
    testing::{handle, init, mock_env, mock_instance, query},
    Api, Instance,
};

use mock::mock_dependencies;
use reflect::msg::{CustomMsg, CustomResponse, HandleMsg, InitMsg, OwnerResponse, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/reflect.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

mod mock {
    use reflect::msg::{CustomQuery, CustomResponse};

    use cosmwasm_std::{from_slice, to_binary, Binary, Coin, QueryRequest, StdResult, SystemError};
    use cosmwasm_vm::{
        testing::{
            mock_dependencies as original_mock_dependencies, MockApi, MockQuerier, MockStorage,
        },
        Extern, Querier, QuerierResult,
    };

    #[derive(Clone)]
    pub struct CustomQuerier {
        base: MockQuerier,
    }

    impl CustomQuerier {
        pub fn new(base: MockQuerier) -> Self {
            CustomQuerier { base }
        }
    }

    impl Querier for CustomQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            // parse into our custom query class
            let request: QueryRequest<CustomQuery> = match from_slice(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Err(SystemError::InvalidRequest {
                        error: format!("Parsing query request: {}", e),
                        request: Binary(bin_request.to_vec()),
                    }))
                }
            };
            if let QueryRequest::Custom(custom_query) = &request {
                Ok(Ok(execute(&custom_query)))
            } else {
                self.base.handle_query(&request)
            }
        }
    }

    /// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
    /// this uses our CustomQuerier.
    pub fn mock_dependencies(
        canonical_length: usize,
        contract_balance: &[Coin],
    ) -> Extern<MockStorage, MockApi, CustomQuerier> {
        let base = original_mock_dependencies(canonical_length, contract_balance);
        base.change_querier(CustomQuerier::new)
    }

    fn execute(query: &CustomQuery) -> StdResult<Binary> {
        let msg = match query {
            CustomQuery::Ping {} => "pong".to_string(),
            CustomQuery::Capital { text } => text.to_uppercase(),
        };
        to_binary(&CustomResponse { msg })
    }
}

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
        Err(StdError::Unauthorized { .. }) => {}
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
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn dispatch_custom_query() {
    // stub gives us defaults. Consume it and override...
    let custom = mock_dependencies(20, &[]);
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
