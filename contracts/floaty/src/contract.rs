use cosmwasm_std::{
    entry_point, from_slice, to_binary, to_vec, AllBalanceResponse, BankMsg, Deps, DepsMut, Env,
    Event, MessageInfo, QueryResponse, Response, StdError, StdResult,
};
use rand_chacha::rand_core::SeedableRng;

#[cfg(target_arch = "wasm32")]
use crate::instructions::run_instruction;
use crate::{
    instructions::{random_args_for, Value, FLOAT_INSTRUCTIONS},
    msg::QueryMsg,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, String> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, String> {
    Ok(Response::default())
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::RandomArgsFor { instruction, seed } => {
            let mut rng = rand_chacha::ChaChaRng::seed_from_u64(seed);
            to_binary(&random_args_for(&instruction, &mut rng))
        }
        QueryMsg::Instructions {} => to_binary(&FLOAT_INSTRUCTIONS.to_vec()),
        QueryMsg::Run { instruction, args } => to_binary(&query_run(&instruction, args)?),
    }
}

fn query_run(instruction: &str, args: Vec<Value>) -> StdResult<Value> {
    #[cfg(not(target_arch = "wasm32"))]
    panic!();

    #[cfg(target_arch = "wasm32")]
    {
        let result = run_instruction(instruction, &args);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balances, mock_env, mock_info, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::Api as _;
    // import trait Storage to get access to read
    use cosmwasm_std::{attr, coins, Addr, Storage, SubMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let expected_state = State {
            verifier: deps.api.addr_validate(&verifier).unwrap(),
            beneficiary: deps.api.addr_validate(&beneficiary).unwrap(),
            funder: deps.api.addr_validate(&creator).unwrap(),
        };

        let msg = InstantiateMsg {
            verifier,
            beneficiary,
        };
        let info = mock_info(creator.as_str(), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
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
    fn instantiate_and_query() {
        let mut deps = mock_dependencies();

        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");
        let creator = String::from("creator");
        let msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary,
        };
        let info = mock_info(&creator, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now let's query
        let query_response = query_verifier(deps.as_ref()).unwrap();
        assert_eq!(query_response.verifier, verifier);
    }

    #[test]
    fn querier_callbacks_work() {
        let rich_addr = String::from("foobar");
        let rich_balance = coins(10000, "gold");
        let deps = mock_dependencies_with_balances(&[(&rich_addr, &rich_balance)]);

        // querying with balance gets the balance
        let bal = query_other_balance(deps.as_ref(), rich_addr).unwrap();
        assert_eq!(bal.amount, rich_balance);

        // querying other accounts gets none
        let bal = query_other_balance(deps.as_ref(), String::from("someone else")).unwrap();
        assert_eq!(bal.amount, vec![]);
    }

    #[test]
    fn execute_release_works() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = String::from("creator");
        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_info = mock_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary can release it
        let execute_info = mock_info(verifier.as_str(), &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {},
        )
        .unwrap();
        assert_eq!(execute_res.messages.len(), 1);
        let msg = execute_res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: beneficiary,
                amount: coins(1000, "earth"),
            }),
        );
        assert_eq!(
            execute_res.attributes,
            vec![
                attr("action", "release"),
                attr("destination", "benefits"),
                attr("foo", "300")
            ],
        );
        assert_eq!(execute_res.data, Some(vec![0xF0, 0x0B, 0xAA].into()));
    }

    #[test]
    fn execute_release_fails_for_wrong_sender() {
        let mut deps = mock_dependencies();

        // initialize the store
        let creator = String::from("creator");
        let verifier = String::from("verifies");
        let beneficiary = String::from("benefits");

        let instantiate_msg = InstantiateMsg {
            verifier: verifier.clone(),
            beneficiary: beneficiary.clone(),
        };
        let init_amount = coins(1000, "earth");
        let init_info = mock_info(&creator, &init_amount);
        let init_res = instantiate(deps.as_mut(), mock_env(), init_info, instantiate_msg).unwrap();
        assert_eq!(init_res.messages.len(), 0);

        // balance changed in init
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, init_amount);

        // beneficiary cannot release it
        let execute_info = mock_info(beneficiary.as_str(), &[]);
        let execute_res = execute(
            deps.as_mut(),
            mock_env(),
            execute_info,
            ExecuteMsg::Release {},
        );
        assert_eq!(execute_res.unwrap_err(), HackError::Unauthorized {});

        // state should not change
        let data = deps.storage.get(CONFIG_KEY).expect("no data stored");
        let state: State = from_slice(&data).unwrap();
        assert_eq!(
            state,
            State {
                verifier: Addr::unchecked(verifier),
                beneficiary: Addr::unchecked(beneficiary),
                funder: Addr::unchecked(creator),
            }
        );
    }
}
