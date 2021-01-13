use cosmwasm_std::{
    attr, entry_point, DepsMut, Env, HandleResponse, IbcChannel, IbcOrder, InitResponse,
    MessageInfo, StdError, StdResult,
};

use crate::msg::{HandleMsg, InitMsg};
use crate::state::{config, Config};

const IBC_VERSION: &str = "ibc-reflect";

#[entry_point]
pub fn init(deps: DepsMut, _env: Env, _info: MessageInfo, msg: InitMsg) -> StdResult<InitResponse> {
    // we store the reflect_id for creating accounts later
    let cfg = Config {
        reflect_code_id: msg.reflect_code_id,
    };
    config(deps.storage).save(&cfg)?;

    Ok(InitResponse::default())
}

#[entry_point]
pub fn handle(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> {
    Err(StdError::generic_err(
        "You can only call this contract via ibc packets",
    ))
}

#[entry_point]
pub fn ibc_channel_open(_deps: DepsMut, _env: Env, msg: IbcChannel) -> StdResult<()> {
    if msg.order != IbcOrder::Ordered {
        return Err(StdError::generic_err("Only supports ordered channels"));
    }
    if msg.version.as_str() != IBC_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            IBC_VERSION
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, HumanAddr, StdError, Storage};

    #[test]
    fn init_fails() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "You can only use this contract for migrations")
            }
            _ => panic!("expected migrate error message"),
        }
    }

    #[test]
    fn migrate_cleans_up_data() {
        let mut deps = mock_dependencies(&coins(123456, "gold"));

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(3, cnt);

        // change the verifier via migrate
        let payout = HumanAddr::from("someone else");
        let msg = MigrateMsg {
            payout: payout.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = migrate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // check payout
        assert_eq!(1, res.messages.len());
        let msg = res.messages.get(0).expect("no message");
        assert_eq!(
            msg,
            &BankMsg::Send {
                to_address: payout,
                amount: coins(123456, "gold"),
            }
            .into(),
        );

        // check there is no data in storage
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(0, cnt);
    }
}
