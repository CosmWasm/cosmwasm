use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryInto;

use cosmwasm_std::{
    from_slice, to_binary, to_vec, AllBalanceResponse, Api, BankMsg, Binary, CanonicalAddr,
    Context, Env, Extern, HandleResponse, HumanAddr, InitResponse, MigrateResponse, Querier,
    QueryRequest, QueryResponse, StdError, StdResult, Storage, WasmQuery,
};

use crate::errors::HackError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub verifier: HumanAddr,
    pub beneficiary: HumanAddr,
}

/// MigrateMsg allows a priviledged contract administrator to run
/// a migration on the contract. In this (demo) case it is just migrating
/// from one hackatom code to the same code, but taking advantage of the
/// migration step to set a new validator.
///
/// Note that the contract doesn't enforce permissions here, this is done
/// by blockchain logic (in the future by blockchain governance)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub verifier: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub verifier: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub funder: CanonicalAddr,
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Releasing all funds in the contract to the beneficiary. This is the only "proper" action of this demo contract.
    Release {},
    /// Infinite loop to burn cpu cycles (only run when metering is enabled)
    CpuLoop {},
    /// Infinite loop making storage calls (to test when their limit hits)
    StorageLoop {},
    /// Infinite loop reading and writing memory
    MemoryLoop {},
    /// Allocate large amounts of memory without consuming much gas
    AllocateLargeMemory {},
    /// Trigger a panic to ensure framework handles gracefully
    Panic {},
    /// Starting with CosmWasm 0.10, some API calls return user errors back to the contract.
    /// This triggers such user errors, ensuring the transaction does not fail in the backend.
    UserErrorsInApiCalls {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// returns a human-readable representation of the verifier
    /// use to ensure query path works in integration tests
    Verifier {},
    /// This returns cosmwasm_std::AllBalanceResponse to demo use of the querier
    OtherBalance { address: HumanAddr },
    /// Recurse will execute a query into itself up to depth-times and return
    /// Each step of the recursion may perform some extra work to test gas metering
    /// (`work` rounds of sha256 on contract).
    /// Contract should be the set to be the address of the original contract,
    /// we pass it in as query doesn't have access to env.
    Recurse {
        depth: u32,
        work: u32,
        contract: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VerifierResponse {
    pub verifier: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RecurseResponse {
    /// hashed is the result of running sha256 "work+1" times on the contract's human address
    pub hashed: Binary,
}

pub const CONFIG_KEY: &[u8] = b"config";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> Result<InitResponse, HackError> {
    deps.api.debug("here we go 🚀");

    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&State {
            verifier: deps.api.canonical_address(&msg.verifier)?,
            beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
            funder: deps.api.canonical_address(&env.message.sender)?,
        })?,
    );

    // This adds some unrelated event attribute for testing purposes
    let mut ctx = Context::new();
    ctx.add_attribute("Let the", "hacking begin");
    Ok(ctx.try_into()?)
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<MigrateResponse, HackError> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let mut config: State = from_slice(&data)?;
    config.verifier = deps.api.canonical_address(&msg.verifier)?;
    deps.storage.set(CONFIG_KEY, &to_vec(&config)?);

    Ok(MigrateResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> Result<HandleResponse, HackError> {
    match msg {
        HandleMsg::Release {} => do_release(deps, env),
        HandleMsg::CpuLoop {} => do_cpu_loop(),
        HandleMsg::StorageLoop {} => do_storage_loop(deps),
        HandleMsg::MemoryLoop {} => do_memory_loop(),
        HandleMsg::AllocateLargeMemory {} => do_allocate_large_memory(),
        HandleMsg::Panic {} => do_panic(),
        HandleMsg::UserErrorsInApiCalls {} => do_user_errors_in_api_calls(&deps.api),
    }
}

fn do_release<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> Result<HandleResponse, HackError> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let state: State = from_slice(&data)?;

    if deps.api.canonical_address(&env.message.sender)? == state.verifier {
        let to_addr = deps.api.human_address(&state.beneficiary)?;
        let balance = deps.querier.query_all_balances(&env.contract.address)?;

        let mut ctx = Context::new();
        ctx.add_attribute("action", "release");
        ctx.add_attribute("destination", &to_addr);
        ctx.add_message(BankMsg::Send {
            from_address: env.contract.address,
            to_address: to_addr,
            amount: balance,
        });
        ctx.set_data(&[0xF0, 0x0B, 0xAA]);
        Ok(ctx.into())
    } else {
        Err(HackError::Unauthorized {})
    }
}

fn do_cpu_loop() -> Result<HandleResponse, HackError> {
    let mut counter = 0u64;
    loop {
        counter += 1;
        if counter >= 9_000_000_000 {
            counter = 0;
        }
    }
}

fn do_storage_loop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> Result<HandleResponse, HackError> {
    let mut test_case = 0u64;
    loop {
        deps.storage
            .set(b"test.key", test_case.to_string().as_bytes());
        test_case += 1;
    }
}

fn do_memory_loop() -> Result<HandleResponse, HackError> {
    let mut data = vec![1usize];
    loop {
        // add one element
        data.push((*data.last().expect("must not be empty")) + 1);
    }
}

fn do_allocate_large_memory() -> Result<HandleResponse, HackError> {
    // We create memory pages explicitely since Rust's default allocator seems to be clever enough
    // to not grow memory for unused capacity like `Vec::<u8>::with_capacity(100 * 1024 * 1024)`.
    // Even with std::alloc::alloc the memory did now grow beyond 1.5 MiB.

    #[cfg(target_arch = "wasm32")]
    {
        use core::arch::wasm32;
        let pages = 1_600; // 100 MiB
        let ptr = wasm32::memory_grow(0, pages);
        if ptr == usize::max_value() {
            return Err(StdError::generic_err("Error in memory.grow instruction").into());
        }
        Ok(HandleResponse::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    Err(StdError::generic_err("Unsupported architecture").into())
}

fn do_panic() -> Result<HandleResponse, HackError> {
    panic!("This page intentionally faulted");
}

fn do_user_errors_in_api_calls<A: Api>(api: &A) -> Result<HandleResponse, HackError> {
    // Canonicalize

    let empty = HumanAddr::from("");
    match api.canonical_address(&empty).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let invalid_bech32 = HumanAddr::from("bn93hg934hg08q340g8u4jcau3");
    match api.canonical_address(&invalid_bech32).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    // Humanize

    let empty: CanonicalAddr = vec![].into();
    match api.human_address(&empty).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let too_short: CanonicalAddr = vec![0xAA, 0xBB, 0xCC].into();
    match api.human_address(&too_short).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    let wrong_length: CanonicalAddr = vec![0xA6; 17].into();
    match api.human_address(&wrong_length).unwrap_err() {
        StdError::GenericErr { .. } => {}
        err => {
            return Err(StdError::generic_err(format!(
                "Unexpected error in do_user_errors_in_api_calls: {:?}",
                err
            ))
            .into())
        }
    }

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verifier {} => to_binary(&query_verifier(deps)?),
        QueryMsg::OtherBalance { address } => to_binary(&query_other_balance(deps, address)?),
        QueryMsg::Recurse {
            depth,
            work,
            contract,
        } => to_binary(&query_recurse(deps, depth, work, contract)?),
    }
}

fn query_verifier<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<VerifierResponse> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("State"))?;
    let state: State = from_slice(&data)?;
    let addr = deps.api.human_address(&state.verifier)?;
    Ok(VerifierResponse { verifier: addr })
}

fn query_other_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<AllBalanceResponse> {
    let amount = deps.querier.query_all_balances(address)?;
    Ok(AllBalanceResponse { amount })
}

fn query_recurse<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    depth: u32,
    work: u32,
    contract: HumanAddr,
) -> StdResult<RecurseResponse> {
    // perform all hashes as requested
    let mut hashed: Vec<u8> = contract.as_str().as_bytes().to_vec();
    for _ in 0..work {
        hashed = Sha256::digest(&hashed).to_vec()
    }

    // the last contract should return the response
    if depth == 0 {
        Ok(RecurseResponse {
            hashed: hashed.into(),
        })
    } else {
        // otherwise, we go one level deeper and return the response of the next level
        let req = QueryMsg::Recurse {
            depth: depth - 1,
            work,
            contract: contract.clone(),
        };
        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract,
            msg: to_binary(&req)?,
        });
        deps.querier.query(&query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balances, mock_env, MOCK_CONTRACT_ADDR,
    };
    // import trait ReadonlyStorage to get access to read
    use cosmwasm_std::{attr, coins, ReadonlyStorage};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));
        let expected_state = State {
            verifier: deps.api.canonical_address(&verifier).unwrap(),
            beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
            funder: deps.api.canonical_address(&creator).unwrap(),
        };

        let msg = InitMsg {
            verifier,
            beneficiary,
        };
        let env = mock_env(creator.as_str(), &[]);
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0].key, "Let the");
        assert_eq!(res.attributes[0].value, "hacking begin");

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(state, expected_state);
    }

    #[test]
    fn init_and_query() {
        let mut deps = mock_dependencies(20, &[]);

        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));
        let msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary,
        };
        let env = mock_env(creator.as_str(), &[]);
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let query_response = query_verifier(&deps).unwrap();
        assert_eq!(query_response.verifier, verifier);
    }

    #[test]
    fn migrate_verifier() {
        let mut deps = mock_dependencies(20, &[]);

        let verifier = HumanAddr::from("verifies");
        let beneficiary = HumanAddr::from("benefits");
        let creator = HumanAddr::from("creator");
        let msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary,
        };
        let env = mock_env(creator.as_str(), &[]);
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'verifies'
        let query_response = query(&deps, QueryMsg::Verifier {}).unwrap();
        assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");

        // change the verifier via migrate
        let new_verifier = HumanAddr::from("someone else");
        let msg = MigrateMsg {
            verifier: new_verifier.clone(),
        };
        let env = mock_env(creator.as_str(), &[]);
        let res = migrate(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'someone else'
        let query_response = query_verifier(&deps).unwrap();
        assert_eq!(query_response.verifier, new_verifier);
    }

    #[test]
    fn querier_callbacks_work() {
        let rich_addr = HumanAddr::from("foobar");
        let rich_balance = coins(10000, "gold");
        let deps = mock_dependencies_with_balances(20, &[(&rich_addr, &rich_balance)]);

        // querying with balance gets the balance
        let bal = query_other_balance(&deps, rich_addr).unwrap();
        assert_eq!(bal.amount, rich_balance);

        // querying other accounts gets none
        let bal = query_other_balance(&deps, HumanAddr::from("someone else")).unwrap();
        assert_eq!(bal.amount, vec![]);
    }

    #[test]
    fn handle_release_works() {
        let mut deps = mock_dependencies(20, &[]);

        // initialize the store
        let creator = HumanAddr::from("creator");
        let verifier = HumanAddr::from("verifies");
        let beneficiary = HumanAddr::from("benefits");

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_env = mock_env(creator.as_str(), &init_amount);
        let contract_addr = init_env.contract.address.clone();
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(&contract_addr, init_amount);

        // beneficiary can release it
        let handle_env = mock_env(verifier.as_str(), &[]);
        let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {}).unwrap();
        assert_eq!(handle_res.messages.len(), 1);
        let msg = handle_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: beneficiary,
                amount: coins(1000, "earth"),
            }
            .into(),
        );
        assert_eq!(
            handle_res.attributes,
            vec![attr("action", "release"), attr("destination", "benefits")],
        );
        assert_eq!(handle_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
    }

    #[test]
    fn handle_release_fails_for_wrong_sender() {
        let mut deps = mock_dependencies(20, &[]);

        // initialize the store
        let creator = HumanAddr::from("creator");
        let verifier = HumanAddr::from("verifies");
        let beneficiary = HumanAddr::from("benefits");

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_env = mock_env(creator.as_str(), &init_amount);
        let contract_addr = init_env.contract.address.clone();
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(&contract_addr, init_amount);

        // beneficiary cannot release it
        let handle_env = mock_env(beneficiary.as_str(), &[]);
        let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {});
        match handle_res.unwrap_err() {
            HackError::Unauthorized { .. } => {}
            _ => panic!("Expect unauthorized error"),
        }

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: deps.api.canonical_address(&verifier).unwrap(),
                beneficiary: deps.api.canonical_address(&beneficiary).unwrap(),
                funder: deps.api.canonical_address(&creator).unwrap(),
            }
        );
    }

    #[test]
    #[should_panic(expected = "This page intentionally faulted")]
    fn handle_panic() {
        let mut deps = mock_dependencies(20, &[]);

        // initialize the store
        let verifier = HumanAddr(String::from("verifies"));
        let beneficiary = HumanAddr(String::from("benefits"));
        let creator = HumanAddr(String::from("creator"));

        let init_msg = InitMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_env = mock_env(creator.as_str(), &coins(1000, "earth"));
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(beneficiary.as_str(), &[]);
        // this should panic
        let _ = handle(&mut deps, handle_env, HandleMsg::Panic {});
    }

    #[test]
    fn handle_user_errors_in_api_calls() {
        let mut deps = mock_dependencies(20, &[]);

        let init_msg = InitMsg {
            verifier: HumanAddr::from("verifies"),
            beneficiary: HumanAddr::from("benefits"),
        };
        let init_env = mock_env("creator", &coins(1000, "earth"));
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env("anyone", &[]);
        handle(&mut deps, handle_env, HandleMsg::UserErrorsInApiCalls {}).unwrap();
    }

    #[test]
    fn query_recursive() {
        // the test framework doesn't handle contracts querying contracts yet,
        // let's just make sure the last step looks right

        let deps = mock_dependencies(20, &[]);
        let contract = HumanAddr::from("my-contract");
        let bin_contract: &[u8] = b"my-contract";

        // return the unhashed value here
        let no_work_query = query_recurse(&deps, 0, 0, contract.clone()).unwrap();
        assert_eq!(no_work_query.hashed, Binary::from(bin_contract));

        // let's see if 5 hashes are done right
        let mut expected_hash = Sha256::digest(bin_contract);
        for _ in 0..4 {
            expected_hash = Sha256::digest(&expected_hash);
        }
        let work_query = query_recurse(&deps, 0, 5, contract).unwrap();
        assert_eq!(work_query.hashed, expected_hash.to_vec().into());
    }
}
