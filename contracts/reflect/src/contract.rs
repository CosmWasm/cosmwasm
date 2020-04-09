use snafu::ResultExt;

use cosmwasm::errors::{contract_err, unauthorized, Result, SerializeErr};
use cosmwasm::serde::to_vec;
use cosmwasm::traits::{Api, Extern, Storage};
use cosmwasm::types::{log, CosmosMsg, Env, HumanAddr, Response};

use crate::msg::{HandleMsg, InitMsg, OwnerResponse, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    _msg: InitMsg,
) -> Result<Response> {
    let state = State {
        owner: env.message.signer,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: HandleMsg,
) -> Result<Response> {
    match msg {
        HandleMsg::ReflectMsg { msgs } => try_reflect(deps, env, msgs),
        HandleMsg::ChangeOwner { owner } => try_change_owner(deps, env, owner),
    }
}

pub fn try_reflect<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msgs: Vec<CosmosMsg>,
) -> Result<Response> {
    let state = config(&mut deps.storage).load()?;
    if env.message.signer != state.owner {
        return unauthorized();
    }
    if msgs.is_empty() {
        return contract_err("Must reflect at least one message");
    }
    let res = Response {
        messages: msgs,
        log: vec![log("action", "reflect")],
        data: None,
    };
    Ok(res)
}

pub fn try_change_owner<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    owner: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.owner = api.canonical_address(&owner)?;
        Ok(state)
    })?;
    Ok(Response {
        log: vec![log("action", "change_owner"), log("owner", owner.as_str())],
        ..Response::default()
    })
}

pub fn query<S: Storage, A: Api>(deps: &Extern<S, A>, msg: QueryMsg) -> Result<Vec<u8>> {
    match msg {
        QueryMsg::Owner {} => query_owner(deps),
    }
}

fn query_owner<S: Storage, A: Api>(deps: &Extern<S, A>) -> Result<Vec<u8>> {
    let state = config_read(&deps.storage).load()?;

    let resp = OwnerResponse {
        owner: deps.api.human_address(&state.owner)?,
    };
    to_vec(&resp).context(SerializeErr {
        kind: "OwnerResponse",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm::encoding::Binary;
    use cosmwasm::errors::Error;
    use cosmwasm::mock::{dependencies, mock_env};
    use cosmwasm::serde::from_slice;
    use cosmwasm::types::coin;

    #[test]
    fn proper_initialization() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(&deps.api, "creator", &coin("1000", "earth"), &[]);

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Owner {}).unwrap();
        let value: OwnerResponse = from_slice(&res).unwrap();
        assert_eq!("creator", value.owner.as_str());
    }

    #[test]
    fn reflect() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
        let payload = vec![CosmosMsg::Send {
            from_address: deps.api.human_address(&env.contract.address).unwrap(),
            to_address: HumanAddr::from("friend"),
            amount: coin("1", "token"),
        }];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn reflect_requires_owner() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        // signer is not owner
        let env = mock_env(&deps.api, "someone", &[], &coin("2", "token"));
        let payload = vec![CosmosMsg::Send {
            from_address: deps.api.human_address(&env.contract.address).unwrap(),
            to_address: HumanAddr::from("friend"),
            amount: coin("1", "token"),
        }];
        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };

        let res = handle(&mut deps, env, msg);
        match res {
            Err(Error::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    #[test]
    fn reflect_reject_empty_msgs() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
        let payload = vec![];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(Error::ContractErr { msg, .. }) => {
                assert_eq!(msg, "Must reflect at least one message")
            }
            _ => panic!("Must return contract error"),
        }
    }

    #[test]
    fn reflect_multiple_messages() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
        let payload = vec![
            CosmosMsg::Send {
                from_address: deps.api.human_address(&env.contract.address).unwrap(),
                to_address: HumanAddr::from("friend"),
                amount: coin("1", "token"),
            },
            CosmosMsg::Opaque {
                data: Binary(b"{\"foo\":123}".to_vec()),
            },
        ];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn transfer() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env(&deps.api, "creator", &[], &coin("2", "token"));
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // should change state
        assert_eq!(0, res.messages.len());
        let res = query(&deps, QueryMsg::Owner {}).unwrap();
        let value: OwnerResponse = from_slice(&res).unwrap();
        assert_eq!("friend", value.owner.as_str());
    }

    #[test]
    fn transfer_requires_owner() {
        let mut deps = dependencies(20);

        let msg = InitMsg {};
        let env = mock_env(
            &deps.api,
            "creator",
            &coin("2", "token"),
            &coin("2", "token"),
        );
        let _res = init(&mut deps, env, msg).unwrap();

        let env = mock_env(&deps.api, "random", &[], &coin("2", "token"));
        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };

        let res = handle(&mut deps, env, msg);
        match res {
            Err(Error::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }
}
