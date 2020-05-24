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
    coin, from_binary, Decimal, HumanAddr, InitResponse, StdError, StdResult, Uint128, Validator,
};
use cosmwasm_vm::testing::{init, mock_dependencies, mock_env, query};
use cosmwasm_vm::Instance;

use staking::msg::{
    BalanceResponse, ClaimsResponse, InitMsg, InvestmentResponse, QueryMsg, TokenInfoResponse,
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/staking.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

fn sample_validator<U: Into<HumanAddr>>(addr: U) -> Validator {
    Validator {
        address: addr.into(),
        commission: Decimal::percent(3),
        max_commission: Decimal::percent(10),
        max_change_rate: Decimal::percent(1),
    }
}

#[test]
fn initialization_with_missing_validator() {
    let mut ext = mock_dependencies(20, &[]);
    ext.querier
        .with_staking("stake", &[sample_validator("john")], &[]);
    let mut deps = Instance::from_code(WASM, ext, 500_000).unwrap();

    let creator = HumanAddr::from("creator");
    let msg = InitMsg {
        name: "Cool Derivative".to_string(),
        symbol: "DRV".to_string(),
        decimals: 9,
        validator: HumanAddr::from("my-validator"),
        exit_tax: Decimal::percent(2),
        min_withdrawal: Uint128(50),
    };
    let env = mock_env(&deps.api, &creator, &[]);

    // make sure we can init with this
    let res: StdResult<InitResponse> = init(&mut deps, env, msg.clone());
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "my-validator is not in the current validator set")
        }
        _ => panic!("expected unregistered validator error"),
    }
}

#[test]
fn proper_initialization() {
    // we need to use the verbose approach here to customize the querier with staking info
    let mut ext = mock_dependencies(20, &[]);
    ext.querier.with_staking(
        "stake",
        &[
            sample_validator("john"),
            sample_validator("mary"),
            sample_validator("my-validator"),
        ],
        &[],
    );
    let mut deps = Instance::from_code(WASM, ext, 500_000).unwrap();

    let creator = HumanAddr::from("creator");
    let msg = InitMsg {
        name: "Cool Derivative".to_string(),
        symbol: "DRV".to_string(),
        decimals: 9,
        validator: HumanAddr::from("my-validator"),
        exit_tax: Decimal::percent(2),
        min_withdrawal: Uint128(50),
    };
    let env = mock_env(&deps.api, &creator, &[]);

    // make sure we can init with this
    let res: InitResponse = init(&mut deps, env, msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // token info is proper
    let res = query(&mut deps, QueryMsg::TokenInfo {}).unwrap();
    let token: TokenInfoResponse = from_binary(&res).unwrap();
    assert_eq!(&token.name, &msg.name);
    assert_eq!(&token.symbol, &msg.symbol);
    assert_eq!(token.decimals, msg.decimals);

    // no balance
    let res = query(
        &mut deps,
        QueryMsg::Balance {
            address: creator.clone(),
        },
    )
    .unwrap();
    let bal: BalanceResponse = from_binary(&res).unwrap();
    assert_eq!(bal.balance, Uint128(0));

    // no claims
    let res = query(
        &mut deps,
        QueryMsg::Claims {
            address: creator.clone(),
        },
    )
    .unwrap();
    let claim: ClaimsResponse = from_binary(&res).unwrap();
    assert_eq!(claim.claims, Uint128(0));

    // investment info correct
    let res = query(&mut deps, QueryMsg::Investment {}).unwrap();
    let invest: InvestmentResponse = from_binary(&res).unwrap();
    assert_eq!(&invest.owner, &creator);
    assert_eq!(&invest.validator, &msg.validator);
    assert_eq!(invest.exit_tax, msg.exit_tax);
    assert_eq!(invest.min_withdrawal, msg.min_withdrawal);

    assert_eq!(invest.token_supply, Uint128(0));
    assert_eq!(invest.staked_tokens, coin(0, "stake"));
    assert_eq!(invest.nominal_value, Decimal::one());
}

// #[test]
// fn increment() {
//     let mut deps = mock_instance(WASM, &coins(2, "token"));
//
//     let msg = InitMsg { count: 17 };
//     let env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let _res: InitResponse = init(&mut deps, env, msg).unwrap();
//
//     // beneficiary can release it
//     let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
//     let msg = HandleMsg::Increment {};
//     let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();
//
//     // should increase counter by 1
//     let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
//     let value: CountResponse = from_binary(&res).unwrap();
//     assert_eq!(18, value.count);
// }
//
// #[test]
// fn reset() {
//     let mut deps = mock_instance(WASM, &coins(2, "token"));
//
//     let msg = InitMsg { count: 17 };
//     let env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let _res: InitResponse = init(&mut deps, env, msg).unwrap();
//
//     // beneficiary can release it
//     let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
//     let msg = HandleMsg::Reset { count: 5 };
//     let res: HandleResult = handle(&mut deps, unauth_env, msg);
//     match res.unwrap_err() {
//         StdError::Unauthorized { .. } => {}
//         _ => panic!("Expected unauthorized"),
//     }
//
//     // only the original creator can reset the counter
//     let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
//     let msg = HandleMsg::Reset { count: 5 };
//     let _res: HandleResponse = handle(&mut deps, auth_env, msg).unwrap();
//
//     // should now be 5
//     let res = query(&mut deps, QueryMsg::GetCount {}).unwrap();
//     let value: CountResponse = from_binary(&res).unwrap();
//     assert_eq!(5, value.count);
// }
