use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, generic_err, log, not_found, to_binary, to_vec, unauthorized, AllBalanceResponse,
    Api, BankMsg, Binary, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResponse, StdResult, Storage,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub verifier: HumanAddr,
    pub beneficiary: HumanAddr,
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
    // Infinite loop to burn cpu cycles (only run when metering is enabled)
    CpuLoop {},
    // Infinite loop making storage calls (to test when their limit hits)
    StorageLoop {},
    /// Infinite loop reading and writing memory
    MemoryLoop {},
    /// Allocate large amounts of memory without consuming much gas
    AllocateLargeMemory {},
    // Trigger a panic to ensure framework handles gracefully
    Panic {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // returns a human-readable representation of the verifier
    // use to ensure query path works in integration tests
    Verifier {},
    // This returns cosmwasm_std::AllBalanceResponse to demo use of the querier
    OtherBalance { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VerifierResponse {
    pub verifier: HumanAddr,
}

pub static CONFIG_KEY: &[u8] = b"config";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    deps.storage.set(
        CONFIG_KEY,
        &to_vec(&State {
            verifier: deps.api.canonical_address(&msg.verifier)?,
            beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
            funder: env.message.sender,
        })?,
    );
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Release {} => do_release(deps, env),
        HandleMsg::CpuLoop {} => do_cpu_loop(),
        HandleMsg::StorageLoop {} => do_storage_loop(deps),
        HandleMsg::MemoryLoop {} => do_memory_loop(),
        HandleMsg::AllocateLargeMemory {} => do_allocate_large_memory(),
        HandleMsg::Panic {} => do_panic(),
    }
}

fn do_release<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| not_found("State"))?;
    let state: State = from_slice(&data)?;

    if env.message.sender == state.verifier {
        let to_addr = deps.api.human_address(&state.beneficiary)?;
        let from_addr = deps.api.human_address(&env.contract.address)?;
        let balance = deps.querier.query_all_balances(&from_addr)?;

        let res = HandleResponse {
            log: vec![log("action", "release"), log("destination", &to_addr)],
            messages: vec![BankMsg::Send {
                from_address: from_addr,
                to_address: to_addr,
                amount: balance,
            }
            .into()],
            data: None,
        };
        Ok(res)
    } else {
        Err(unauthorized())
    }
}

fn do_cpu_loop() -> StdResult<HandleResponse> {
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
) -> StdResult<HandleResponse> {
    let mut test_case = 0u64;
    loop {
        deps.storage
            .set(b"test.key", test_case.to_string().as_bytes());
        test_case += 1;
    }
}

fn do_memory_loop() -> StdResult<HandleResponse> {
    let mut data = vec![1usize];
    loop {
        // add one element
        data.push((*data.last().expect("must not be empty")) + 1);
    }
}

fn do_allocate_large_memory() -> StdResult<HandleResponse> {
    // We create memory pages explicitely since Rust's default allocator seems to be clever enough
    // to not grow memory for unused capacity like `Vec::<u8>::with_capacity(100 * 1024 * 1024)`.
    // Even with std::alloc::alloc the memory did now grow beyond 1.5 MiB.

    #[cfg(target_arch = "wasm32")]
    {
        use core::arch::wasm32;
        let pages = 1_600; // 100 MiB
        let ptr = wasm32::memory_grow(0, pages);
        if ptr == usize::max_value() {
            return Err(generic_err("Error in memory.grow instruction"));
        }
        Ok(HandleResponse::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    Err(generic_err("Unsupported architecture"))
}

fn do_panic() -> StdResult<HandleResponse> {
    panic!("This page intentionally faulted");
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verifier {} => query_verifier(deps),
        QueryMsg::OtherBalance { address } => query_other_balance(deps, address),
    }
}

fn query_verifier<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryResponse> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| not_found("State"))?;
    let state: State = from_slice(&data)?;
    let addr = deps.api.human_address(&state.verifier)?;
    Ok(Binary(to_vec(&VerifierResponse { verifier: addr })?))
}

fn query_other_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<QueryResponse> {
    let amount = deps.querier.query_all_balances(address)?;
    to_binary(&AllBalanceResponse { amount })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balances, mock_env, MOCK_CONTRACT_ADDR,
    };
    // import trait ReadonlyStorage to get access to read
    use cosmwasm_std::{coins, from_binary, AllBalanceResponse, ReadonlyStorage, StdError};

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
        let env = mock_env(&deps.api, creator.as_str(), &[]);
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

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
        let env = mock_env(&deps.api, creator.as_str(), &[]);
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let query_response = query(&deps, QueryMsg::Verifier {}).unwrap();
        assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");
    }

    #[test]
    fn querier_callbacks_work() {
        let rich_addr = HumanAddr::from("foobar");
        let rich_balance = coins(10000, "gold");
        let deps = mock_dependencies_with_balances(20, &[(&rich_addr, &rich_balance)]);

        // querying with balance gets the balance
        let query_msg = QueryMsg::OtherBalance { address: rich_addr };
        let query_response = query(&deps, query_msg).unwrap();
        let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
        assert_eq!(bal.amount, rich_balance);

        // querying other accounts gets none
        let query_msg = QueryMsg::OtherBalance {
            address: HumanAddr::from("someone else"),
        };
        let query_response = query(&deps, query_msg).unwrap();
        let bal: AllBalanceResponse = from_binary(&query_response).unwrap();
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
        let init_env = mock_env(&deps.api, creator.as_str(), &init_amount);
        let contract_addr = deps.api.human_address(&init_env.contract.address).unwrap();
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(&contract_addr, init_amount);

        // beneficiary can release it
        let handle_env = mock_env(&deps.api, verifier.as_str(), &[]);
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
            handle_res.log,
            vec![log("action", "release"), log("destination", "benefits"),],
        );
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
        let init_env = mock_env(&deps.api, creator.as_str(), &init_amount);
        let contract_addr = deps.api.human_address(&init_env.contract.address).unwrap();
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(&contract_addr, init_amount);

        // beneficiary cannot release it
        let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[]);
        let handle_res = handle(&mut deps, handle_env, HandleMsg::Release {});
        match handle_res.unwrap_err() {
            StdError::Unauthorized { .. } => {}
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
        let init_env = mock_env(&deps.api, creator.as_str(), &coins(1000, "earth"));
        let init_res = init(&mut deps, init_env, init_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let handle_env = mock_env(&deps.api, beneficiary.as_str(), &[]);
        // this should panic
        let _ = handle(&mut deps, handle_env, HandleMsg::Panic {});
    }
}
