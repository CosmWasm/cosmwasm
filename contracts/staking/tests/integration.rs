//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.

use cosmwasm_std::{coin, from_json, ContractResult, Decimal, Response, Uint128, Validator};
use cosmwasm_vm::testing::{
    instantiate, mock_backend, mock_env, mock_info, mock_instance_options, query,
};
use cosmwasm_vm::Instance;

use staking::msg::{
    BalanceResponse, ClaimsResponse, InstantiateMsg, InvestmentResponse, QueryMsg,
    TokenInfoResponse,
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/staking.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

fn sample_validator(addr: &str) -> Validator {
    Validator::create(
        addr.to_owned(),
        Decimal::percent(3),
        Decimal::percent(10),
        Decimal::percent(1),
    )
}

#[test]
fn initialization_with_missing_validator() {
    let mut backend = mock_backend(&[]);
    backend
        .querier
        .update_staking("ustake", &[sample_validator("john")], &[]);
    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();

    let creator = deps.api().addr_make("creator");
    let msg = InstantiateMsg {
        name: "Cool Derivative".to_string(),
        symbol: "DRV".to_string(),
        decimals: 9,
        validator: String::from("my-validator"),
        exit_tax: Decimal::percent(2),
        min_withdrawal: Uint128::new(50),
    };
    let info = mock_info(&creator, &[]);

    // make sure we can instantiate with this
    let res: ContractResult<Response> = instantiate(&mut deps, mock_env(), info, msg);
    let msg = res.unwrap_err();
    assert_eq!(
        msg,
        "Generic error: my-validator is not in the current validator set"
    );
}

#[test]
fn proper_initialization() {
    // we need to use the verbose approach here to customize the querier with staking info
    let mut backend = mock_backend(&[]);
    backend.querier.update_staking(
        "ustake",
        &[
            sample_validator("john"),
            sample_validator("mary"),
            sample_validator("my-validator"),
        ],
        &[],
    );
    let (instance_options, memory_limit) = mock_instance_options();
    let mut deps = Instance::from_code(WASM, backend, instance_options, memory_limit).unwrap();
    assert_eq!(deps.required_capabilities().len(), 1);
    assert!(deps.required_capabilities().contains("staking"));

    let creator = deps.api().addr_make("creator");
    let msg = InstantiateMsg {
        name: "Cool Derivative".to_string(),
        symbol: "DRV".to_string(),
        decimals: 9,
        validator: String::from("my-validator"),
        exit_tax: Decimal::percent(2),
        min_withdrawal: Uint128::new(50),
    };
    let info = mock_info(&creator, &[]);

    // make sure we can init with this
    let res: Response = instantiate(&mut deps, mock_env(), info, msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());

    // token info is proper
    let res = query(&mut deps, mock_env(), QueryMsg::TokenInfo {}).unwrap();
    let token: TokenInfoResponse = from_json(res).unwrap();
    assert_eq!(&token.name, &msg.name);
    assert_eq!(&token.symbol, &msg.symbol);
    assert_eq!(token.decimals, msg.decimals);

    // no balance
    let res = query(
        &mut deps,
        mock_env(),
        QueryMsg::Balance {
            address: creator.clone(),
        },
    )
    .unwrap();
    let bal: BalanceResponse = from_json(res).unwrap();
    assert_eq!(bal.balance, Uint128::new(0));

    // no claims
    let res = query(
        &mut deps,
        mock_env(),
        QueryMsg::Claims {
            address: creator.clone(),
        },
    )
    .unwrap();
    let claim: ClaimsResponse = from_json(res).unwrap();
    assert_eq!(claim.claims, Uint128::new(0));

    // investment info correct
    let res = query(&mut deps, mock_env(), QueryMsg::Investment {}).unwrap();
    let invest: InvestmentResponse = from_json(res).unwrap();
    assert_eq!(&invest.owner, &creator);
    assert_eq!(&invest.validator, &msg.validator);
    assert_eq!(invest.exit_tax, msg.exit_tax);
    assert_eq!(invest.min_withdrawal, msg.min_withdrawal);

    assert_eq!(invest.token_supply, Uint128::new(0));
    assert_eq!(invest.staked_tokens, coin(0, "ustake"));
    assert_eq!(invest.nominal_value, Decimal::one());
}
