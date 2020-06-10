use cosmwasm_std::{
    generic_err, log, Api, BankMsg, Binary, Env, Extern, HandleResponse, InitResponse,
    MigrateResponse, Order, Querier, StdResult, Storage,
};

use crate::msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Err(generic_err("You can only use this contract for migrations"))
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> {
    Err(generic_err("You can only use this contract for migrations"))
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MigrateMsg,
) -> StdResult<MigrateResponse> {
    // delete all state
    let keys: Vec<_> = deps
        .storage
        .range(None, None, Order::Ascending)?
        .map(|(k, _)| k)
        .collect();
    let count = keys.len();
    for k in keys {
        deps.storage.remove(&k);
    }

    // get balance and send all to recipient
    let from_addr = deps.api.human_address(&env.contract.address)?;
    let balance = deps.querier.query_all_balances(&from_addr)?;
    let send = BankMsg::Send {
        from_address: from_addr,
        to_address: msg.payout.clone(),
        amount: balance,
    };

    let data_msg = format!("burnt {} keys", count).into_bytes();

    Ok(MigrateResponse {
        messages: vec![send.into()],
        log: vec![log("action", "burn"), log("payout", msg.payout)],
        data: Some(data_msg.into()),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    Err(generic_err("You can only use this contract for migrations"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, HumanAddr, ReadonlyStorage, StdError};

    #[test]
    fn init_fails() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {};
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "You can only use this contract for migrations")
            }
            _ => panic!("expected migrate error message"),
        }
    }

    #[test]
    fn migrate_cleans_up_data() {
        let mut deps = mock_dependencies(20, &coins(123456, "gold"));

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps
            .storage
            .range(None, None, Order::Ascending)
            .unwrap()
            .count();
        assert_eq!(3, cnt);

        // change the verifier via migrate
        let payout = HumanAddr::from("someone else");
        let msg = MigrateMsg {
            payout: payout.clone(),
        };
        let env = mock_env(&deps.api, "creator", &[]);
        let res = migrate(&mut deps, env, msg).unwrap();
        // check payout
        assert_eq!(1, res.messages.len());
        let msg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: payout,
                amount: coins(123456, "gold"),
            }
            .into(),
        );

        // check there is no data in storage
        let cnt = deps
            .storage
            .range(None, None, Order::Ascending)
            .unwrap()
            .count();
        assert_eq!(0, cnt);
    }
}
