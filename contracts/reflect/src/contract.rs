use cosmwasm_std::{
    attr, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage,
};

use crate::msg::{
    CustomMsg, CustomQuery, CustomResponse, HandleMsg, InitMsg, OwnerResponse, QueryMsg,
};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse<CustomMsg>> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<CustomMsg>> {
    match msg {
        HandleMsg::ReflectMsg { msgs } => try_reflect(deps, env, msgs),
        HandleMsg::ChangeOwner { owner } => try_change_owner(deps, env, owner),
    }
}

pub fn try_reflect<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msgs: Vec<CosmosMsg<CustomMsg>>,
) -> StdResult<HandleResponse<CustomMsg>> {
    let state = config(&mut deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::unauthorized());
    }
    if msgs.is_empty() {
        return Err(StdError::generic_err("Must reflect at least one message"));
    }
    let res = HandleResponse {
        messages: msgs,
        attributes: vec![attr("action", "reflect")],
        data: None,
    };
    Ok(res)
}

pub fn try_change_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
) -> StdResult<HandleResponse<CustomMsg>> {
    let api = deps.api;
    config(&mut deps.storage).update(|mut state| {
        if api.canonical_address(&env.message.sender)? != state.owner {
            return Err(StdError::unauthorized());
        }
        state.owner = api.canonical_address(&owner)?;
        Ok(state)
    })?;
    Ok(HandleResponse {
        attributes: vec![attr("action", "change_owner"), attr("owner", owner)],
        ..HandleResponse::default()
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::ReflectCustom { text } => to_binary(&query_reflect(deps, text)?),
    }
}

fn query_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<OwnerResponse> {
    let state = config_read(&deps.storage).load()?;
    let resp = OwnerResponse {
        owner: deps.api.human_address(&state.owner)?,
    };
    Ok(resp)
}

fn query_reflect<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    text: String,
) -> StdResult<CustomResponse> {
    let req = CustomQuery::Capital { text }.into();
    deps.querier.custom_query(&req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mock_dependencies_with_custom_querier;
    use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, BankMsg, Binary, StakingMsg, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let value = query_owner(&deps).unwrap();
        assert_eq!("creator", value.owner.as_str());
    }

    #[test]
    fn reflect() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let env = mock_env("creator", &[]);
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn reflect_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // signer is not owner
        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];
        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };

        let env = mock_env("random", &[]);
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn reflect_reject_empty_msgs() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("creator", &[]);
        let payload = vec![];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { msg, .. }) => {
                assert_eq!(msg, "Must reflect at least one message")
            }
            _ => panic!("Must return contract error"),
        }
    }

    #[test]
    fn reflect_multiple_messages() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let payload = vec![
            BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
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
        let env = mock_env("creator", &[]);
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("creator", &[]);
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // should change state
        assert_eq!(0, res.messages.len());
        let value = query_owner(&deps).unwrap();
        assert_eq!("friend", value.owner.as_str());
    }

    #[test]
    fn transfer_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(20, &[]);

        let msg = InitMsg {};
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env("random", &[]);
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };

        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn dispatch_custom_query() {
        let deps = mock_dependencies_with_custom_querier(20, &[]);

        // we don't even initialize, just trigger a query
        let value = query_reflect(&deps, "demo one".to_string()).unwrap();
        assert_eq!(value.msg, "DEMO ONE");
    }
}
