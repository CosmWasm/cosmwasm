use cosmwasm_std::{
    entry_point, BankMsg, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, Storage,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};

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
    // get balance and send all to recipient
    #[allow(deprecated)]
    let balance = deps.querier.query_all_balances(env.contract.address)?;
    let send = BankMsg::Send {
        to_address: msg.payout.clone(),
        amount: balance,
    };

    let deleted = cleanup(deps.storage, msg.delete as usize);

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "migrate")
        .add_attribute("payout", msg.payout)
        .add_attribute("deleted_entries", deleted.to_string()))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Cleanup { limit } => execute_cleanup(deps, env, info, limit),
    }
}

pub fn execute_cleanup(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    limit: Option<u32>,
) -> StdResult<Response> {
    let limit = limit.unwrap_or(u32::MAX) as usize;
    let deleted = cleanup(deps.storage, limit);

    Ok(Response::new()
        .add_attribute("action", "cleanup")
        .add_attribute("deleted_entries", deleted.to_string()))
}

fn cleanup(storage: &mut dyn Storage, mut limit: usize) -> usize {
    let mut deleted = 0;
    const PER_SCAN: usize = 20;
    loop {
        let take_this_scan = std::cmp::min(PER_SCAN, limit);
        let keys: Vec<_> = storage
            .range_keys(None, None, Order::Ascending)
            .take(take_this_scan)
            .collect();
        let deleted_this_scan = keys.len();
        for k in keys {
            storage.remove(&k);
        }
        deleted += deleted_this_scan;
        // decrease the number of elements we can still take
        limit -= deleted_this_scan;
        if limit == 0 || deleted_this_scan < take_this_scan {
            break;
        }
    }

    deleted
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_dependencies_with_balance, mock_env,
    };
    use cosmwasm_std::{coins, Attribute, StdError, Storage, SubMsg};

    /// Gets the value of the first attribute with the given key
    fn first_attr(data: impl AsRef<[Attribute]>, search_key: &str) -> Option<String> {
        data.as_ref().iter().find_map(|a| {
            if a.key == search_key {
                Some(a.value.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn instantiate_fails() {
        let mut deps = mock_dependencies();

        let creator = deps.api.addr_make("creator");

        let msg = InstantiateMsg {};
        let info = message_info(&creator, &coins(1000, "earth"));
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
    fn migrate_sends_funds() {
        let mut deps = mock_dependencies_with_balance(&coins(123456, "gold"));

        // change the verifier via migrate
        let payout = String::from("someone else");
        let msg = MigrateMsg {
            payout: payout.clone(),
            delete: 0,
        };
        let res = migrate(deps.as_mut(), mock_env(), msg).unwrap();
        // check payout
        assert_eq!(1, res.messages.len());
        let msg = res.messages.first().expect("no message");
        assert_eq!(
            msg,
            &SubMsg::new(BankMsg::Send {
                to_address: payout,
                amount: coins(123456, "gold"),
            })
        );
    }

    #[test]
    fn migrate_with_delete() {
        let mut deps = mock_dependencies_with_balance(&coins(123456, "gold"));

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(cnt, 3);

        // migrate all of the data in one go
        let msg = MigrateMsg {
            payout: "user".to_string(),
            delete: 100,
        };
        migrate(deps.as_mut(), mock_env(), msg).unwrap();

        // no more data
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(cnt, 0);
    }

    #[test]
    fn execute_cleans_up_data() {
        let mut deps = mock_dependencies_with_balance(&coins(123456, "gold"));

        let anon = deps.api.addr_make("anon");

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(cnt, 3);

        // change the verifier via migrate
        let payout = String::from("someone else");
        let msg = MigrateMsg { payout, delete: 0 };
        let _res = migrate(deps.as_mut(), mock_env(), msg).unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&anon, &[]),
            ExecuteMsg::Cleanup { limit: Some(2) },
        )
        .unwrap();
        assert_eq!(first_attr(res.attributes, "deleted_entries").unwrap(), "2");

        // One item should be left
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(cnt, 1);

        let res = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&anon, &[]),
            ExecuteMsg::Cleanup { limit: Some(2) },
        )
        .unwrap();
        assert_eq!(first_attr(res.attributes, "deleted_entries").unwrap(), "1");

        // Now all are gone
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(cnt, 0);
    }
}
