use cosmwasm_std::{
    entry_point, BankMsg, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
};

use crate::msg::{InstantiateMsg, MigrateMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Err(StdError::generic_err(
        "You can only use this contract for migrations",
    ))
}

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    // delete all state
    let keys: Vec<_> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(k, _)| k)
        .collect();
    let count = keys.len();
    for k in keys {
        deps.storage.remove(&k);
    }

    // get balance and send all to recipient
    let balance = deps.querier.query_all_balances(env.contract.address)?;
    let send = BankMsg::Send {
        to_address: msg.payout.clone(),
        amount: balance,
    };

    let data_msg = format!("burnt {} keys", count).into_bytes();

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "burn")
        .add_attribute("payout", msg.payout)
        .set_data(data_msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, StdError, Storage, SubMsg};

    #[test]
    fn instantiate_fails() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "You can only use this contract for migrations")
            }
            _ => panic!("expected migrate error message"),
        }
    }

    #[test]
    fn migrate_cleans_up_data() {
        let mut deps = mock_dependencies_with_balance(&coins(123456, "gold"));

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(3, cnt);

        // change the verifier via migrate
        let payout = String::from("someone else");
        let msg = MigrateMsg {
            payout: payout.clone(),
        };
        let res = migrate(deps.as_mut(), mock_env(), msg).unwrap();
        // check payout
        assert_eq!(1, res.messages.len());
        let msg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: payout,
                amount: coins(123456, "gold"),
            })
        );

        // check there is no data in storage
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(0, cnt);
    }
}
