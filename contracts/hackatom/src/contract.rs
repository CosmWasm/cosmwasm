use sha2::{Digest, Sha256};

use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Addr, Api, BankMsg, CanonicalAddr, Deps,
    DepsMut, Env, Event, MessageInfo, MigrateInfo, QueryRequest, QueryResponse, Response, StdError,
    StdErrorKind, StdResult, WasmMsg, WasmQuery,
};

use crate::errors::HackError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, IntResponse, MigrateMsg, QueryMsg, RecurseResponse, SudoMsg,
    VerifierResponse,
};
use crate::state::{State, CONFIG_KEY};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, HackError> {
    deps.api.debug("here we go 🚀");

    deps.storage.set(
        CONFIG_KEY,
        &to_json_vec(&State {
            verifier: deps.api.addr_validate(&msg.verifier)?,
            beneficiary: deps.api.addr_validate(&msg.beneficiary)?,
            funder: info.sender,
        })?,
    );

    // This adds some unrelated event attribute for testing purposes
    Ok(Response::new().add_attribute("Let the", "hacking begin"))
}

const CONTRACT_MIGRATE_VERSION: u64 = 420;

#[entry_point]
#[migrate_version(CONTRACT_MIGRATE_VERSION)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
    migrate_info: MigrateInfo,
) -> Result<Response, HackError> {
    if let Some(old_version) = migrate_info.old_migrate_version {
        if CONTRACT_MIGRATE_VERSION <= old_version {
            return Err(HackError::Downgrade);
        }
    }
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::msg("State not found"))?;
    let mut config: State = from_json(data)?;
    config.verifier = deps.api.addr_validate(&msg.verifier)?;
    deps.storage.set(CONFIG_KEY, &to_json_vec(&config)?);

    Ok(Response::default())
}

#[entry_point]
pub fn sudo(_deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, HackError> {
    match msg {
        SudoMsg::StealFunds { recipient, amount } => {
            let msg = BankMsg::Send {
                to_address: recipient,
                amount,
            };
            Ok(Response::new().add_message(msg))
        }
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, HackError> {
    match msg {
        ExecuteMsg::Release { denom } => do_release(deps, env, info, denom),
        ExecuteMsg::CpuLoop {} => do_cpu_loop(),
        ExecuteMsg::StorageLoop {} => do_storage_loop(deps),
        ExecuteMsg::MemoryLoop {} => do_memory_loop(),
        ExecuteMsg::MessageLoop {} => do_message_loop(env),
        ExecuteMsg::AllocateLargeMemory { pages } => do_allocate_large_memory(pages),
        ExecuteMsg::Panic {} => do_panic(),
        ExecuteMsg::UserErrorsInApiCalls {} => do_user_errors_in_api_calls(deps.api),
    }
}

fn do_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, HackError> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::msg("State not found"))?;
    let state: State = from_json(data)?;

    if info.sender == state.verifier {
        let to_addr = state.beneficiary;
        let balance = deps.querier.query_balance(env.contract.address, denom)?;

        let resp = Response::new()
            .add_attribute("action", "release")
            .add_attribute("destination", to_addr.clone())
            .add_event(Event::new("hackatom").add_attribute("action", "release"))
            .add_message(BankMsg::Send {
                to_address: to_addr.into(),
                amount: vec![balance],
            })
            .set_data([0xF0, 0x0B, 0xAA]);
        Ok(resp)
    } else {
        Err(HackError::Unauthorized {})
    }
}

fn do_cpu_loop() -> Result<Response, HackError> {
    let mut counter = 0u64;
    loop {
        counter += 1;
        if counter >= 9_000_000_000 {
            counter = 0;
        }
    }
}

fn do_storage_loop(deps: DepsMut) -> Result<Response, HackError> {
    let mut test_case = 0u64;
    loop {
        deps.storage
            .set(b"test.key", test_case.to_string().as_bytes());
        test_case += 1;
    }
}

fn do_memory_loop() -> Result<Response, HackError> {
    let mut data = vec![1usize];
    loop {
        // add one element
        data.push((*data.last().expect("must not be empty")) + 1);
    }
}

fn do_message_loop(env: Env) -> Result<Response, HackError> {
    let resp = Response::new().add_message(WasmMsg::Execute {
        contract_addr: env.contract.address.into(),
        msg: to_json_binary(&ExecuteMsg::MessageLoop {})?,
        funds: vec![],
    });
    Ok(resp)
}

#[allow(unused_variables)]
fn do_allocate_large_memory(pages: u32) -> Result<Response, HackError> {
    // We create memory pages explicitly since Rust's default allocator seems to be clever enough
    // to not grow memory for unused capacity like `Vec::<u8>::with_capacity(100 * 1024 * 1024)`.
    // Even with std::alloc::alloc the memory did now grow beyond 1.5 MiB.

    #[cfg(target_arch = "wasm32")]
    {
        use core::arch::wasm32;
        let old_size = wasm32::memory_grow(0, pages as usize);
        if old_size == usize::max_value() {
            return Err(StdError::msg("memory.grow failed").into());
        }
        Ok(Response::new().set_data((old_size as u32).to_be_bytes()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    Err(StdError::msg("Unsupported architecture").into())
}

fn do_panic() -> Result<Response, HackError> {
    // Uncomment your favourite panic case

    // panicked at 'This page intentionally faulted', src/contract.rs:53:5
    panic!("This page intentionally faulted");

    // panicked at 'oh no (a = 3)', src/contract.rs:56:5
    // let a = 3;
    // panic!("oh no (a = {a})");

    // panicked at 'attempt to subtract with overflow', src/contract.rs:59:13
    // #[allow(arithmetic_overflow)]
    // let _ = 5u32 - 8u32;

    // panicked at 'no entry found for key', src/contract.rs:62:13
    // let map = std::collections::HashMap::<String, String>::new();
    // let _ = map["foo"];
}

fn do_user_errors_in_api_calls(api: &dyn Api) -> Result<Response, HackError> {
    // Canonicalize

    let empty = "";
    match api.addr_canonicalize(empty).unwrap_err().kind() {
        StdErrorKind::Parsing => {}
        err => {
            return Err(StdError::msg(format_args!(
                "Unexpected error in do_user_errors_in_api_calls: {err:?}"
            ))
            .into())
        }
    }

    let invalid = "bn9hhssomeltvhzgvuqkwjkpwxoj";
    match api.addr_canonicalize(invalid).unwrap_err().kind() {
        StdErrorKind::Parsing => {}
        err => {
            return Err(StdError::msg(format_args!(
                "Unexpected error in do_user_errors_in_api_calls: {err:?}"
            ))
            .into())
        }
    }

    let too_long = "cosmwasm1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqehqqkz";
    match api.addr_canonicalize(too_long).unwrap_err().kind() {
        StdErrorKind::Parsing => {}
        err => {
            return Err(StdError::msg(format_args!(
                "Unexpected error in do_user_errors_in_api_calls: {err:?}"
            ))
            .into())
        }
    }

    // Humanize
    let empty: CanonicalAddr = vec![].into();
    match api.addr_humanize(&empty).unwrap_err().kind() {
        StdErrorKind::Encoding => {}
        err => {
            return Err(StdError::msg(format_args!(
                "Unexpected error in do_user_errors_in_api_calls: {err:?}"
            ))
            .into())
        }
    }

    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Verifier {} => to_json_binary(&query_verifier(deps)?),
        QueryMsg::Recurse { depth, work } => {
            to_json_binary(&query_recurse(deps, depth, work, env.contract.address)?)
        }
        QueryMsg::GetInt {} => to_json_binary(&query_int()),
    }
}

fn query_verifier(deps: Deps) -> StdResult<VerifierResponse> {
    let data = deps
        .storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::msg("State not found"))?;
    let state: State = from_json(data)?;
    Ok(VerifierResponse {
        verifier: state.verifier.into(),
    })
}

fn query_recurse(deps: Deps, depth: u32, work: u32, contract: Addr) -> StdResult<RecurseResponse> {
    // perform all hashes as requested
    let mut hashed: Vec<u8> = contract.as_str().into();
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
        };
        let query = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract.into(),
            msg: to_json_binary(&req)?,
        });
        deps.querier.query(&query)
    }
}

fn query_int() -> IntResponse {
    IntResponse { int: 0xf00baa }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
    // import trait Storage to get access to read
    use cosmwasm_std::{coin, coins, Binary, Storage, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");
        let creator = deps.api.addr_make("creator");

        let expected_state = State {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
            funder: creator.clone(),
        };

        let msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes, [("Let the", "hacking begin")]);

        // it worked, let's check the state
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_json(data).unwrap();
        assert_eq!(state, expected_state);
    }

    #[test]
    fn instantiate_and_query() {
        let mut deps = mock_dependencies();

        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let query_response = query_verifier(deps.as_ref()).unwrap();
        assert_eq!(query_response.verifier, verifier.as_str());
    }

    #[test]
    fn migrate_verifier() {
        let mut deps = mock_dependencies();

        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'verifies'
        let query_response = query(deps.as_ref(), mock_env(), QueryMsg::Verifier {}).unwrap();
        assert_eq!(
            query_response.as_slice(),
            format!(r#"{{"verifier":"{verifier}"}}"#).as_bytes()
        );

        // change the verifier via migrate
        let new_verifier: String = deps.api.addr_make("someone else").into();
        let msg = MigrateMsg {
            verifier: new_verifier.clone(),
        };
        let migrate_info = MigrateInfo {
            sender: creator,
            old_migrate_version: None,
        };
        let res = migrate(deps.as_mut(), mock_env(), msg, migrate_info).unwrap();
        assert_eq!(0, res.messages.len());

        // check it is 'someone else'
        let query_response = query_verifier(deps.as_ref()).unwrap();
        assert_eq!(query_response.verifier, new_verifier);
    }

    #[test]
    fn sudo_can_steal_tokens() {
        let mut deps = mock_dependencies();

        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");
        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // sudo takes any tax it wants
        let to_address = deps.api.addr_make("community-pool");
        let amount = coins(700, "gold");
        let sys_msg = SudoMsg::StealFunds {
            recipient: to_address.to_string(),
            amount: amount.clone(),
        };
        let res = sudo(deps.as_mut(), mock_env(), sys_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let msg = res.messages.first().expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: to_address.to_string(),
                amount
            })
        );
    }

    #[test]
    fn execute_release_works() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = deps.api.addr_make("creator");
        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let init_amount = coins(1000, "earth");
        let info = message_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier
            .bank
            .update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary can release it
        let execute_info = message_info(&verifier, &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {
                denom: "earth".to_string(),
            },
        )
        .unwrap();
        assert_eq!(execute_res.messages.len(), 1);
        let msg = execute_res.messages.first().expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(1000, "earth"),
            }),
        );
        assert_eq!(
            execute_res.attributes,
            vec![("action", "release"), ("destination", beneficiary.as_str())],
        );
        assert_eq!(execute_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
    }

    #[test]
    fn execute_release_can_be_called_multiple_times() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = deps.api.addr_make("creator");
        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let init_amount = vec![coin(1000, "earth"), coin(70, "sun")];
        let info = message_info(&creator, &init_amount);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        // balance changed in init
        deps.querier
            .bank
            .update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary can release it
        let execute_info = message_info(&verifier, &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {
                denom: "sun".to_string(),
            },
        )
        .unwrap();
        assert_eq!(execute_res.messages.len(), 1);
        let msg = execute_res.messages.first().expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(70, "sun"),
            }),
        );

        // beneficiary can release it again
        let execute_info = message_info(&verifier, &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {
                denom: "earth".to_string(),
            },
        )
        .unwrap();
        assert_eq!(execute_res.messages.len(), 1);
        let msg = execute_res.messages.first().expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: beneficiary.to_string(),
                amount: coins(1000, "earth"),
            }),
        );
    }

    #[test]
    fn execute_release_fails_for_wrong_sender() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = deps.api.addr_make("creator");
        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let init_amount = coins(1000, "earth");
        let info = message_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier
            .bank
            .update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary cannot release it
        let execute_info = message_info(&beneficiary, &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {
                denom: "earth".to_string(),
            },
        );
        assert_eq!(execute_res.unwrap_err().to_string(), "Unauthorized");

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_json(data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: verifier.clone(),
                beneficiary: beneficiary.clone(),
                funder: creator.clone(),
            }
        );
    }

    #[test]
    #[should_panic(expected = "This page intentionally faulted")]
    fn execute_panic() {
        let mut deps = mock_dependencies();

        // initialize the store
        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");
        let creator = deps.api.addr_make("creator");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &coins(1000, "earth"));
        let init_res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, init_res.messages.len());

        let execute_info = message_info(&beneficiary, &[]);
        // this should panic
        let _ = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Panic {},
        );
    }

    #[test]
    fn execute_user_errors_in_api_calls() {
        let mut deps = mock_dependencies();

        let creator = deps.api.addr_make("creator");
        let anyone = deps.api.addr_make("anyone");
        let verifier = deps.api.addr_make("verifies");
        let beneficiary = deps.api.addr_make("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.to_string(),
            beneficiary: beneficiary.to_string(),
        };
        let info = message_info(&creator, &coins(1000, "earth"));
        let response = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, response.messages.len());

        let execute_info = message_info(&anyone, &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::UserErrorsInApiCalls {},
        )
        .unwrap();
    }

    #[test]
    fn query_recursive() {
        // the test framework doesn't handle contracts querying contracts yet,
        // let's just make sure the last step looks right

        let deps = mock_dependencies();
        let contract = Addr::unchecked("my-contract");
        let bin_contract: &[u8] = b"my-contract";

        // return the un-hashed value here
        let no_work_query = query_recurse(deps.as_ref(), 0, 0, contract.clone()).unwrap();
        assert_eq!(no_work_query.hashed, Binary::from(bin_contract));

        // let's see if 5 hashes are done right
        let mut expected_hash = Sha256::digest(bin_contract);
        for _ in 0..4 {
            expected_hash = Sha256::digest(expected_hash);
        }
        let work_query = query_recurse(deps.as_ref(), 0, 5, contract).unwrap();
        assert_eq!(work_query.hashed, expected_hash.to_vec());
    }

    #[test]
    fn get_int() {
        let get_int_query = query_int();
        assert_eq!(get_int_query.int, 0xf00baa);
    }
}
