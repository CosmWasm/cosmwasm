use crate::{Addr, Coin, MessageInfo};

/// A constructor function for [`MessageInfo`].
///
/// This is designed for writing contract tests.
/// It lives in `cosmwasm_std::testing` because constructing MessageInfo
/// objects is not something that you usually need in contract code.
///
/// ## Examples
///
/// ```
/// # use cosmwasm_std::{DepsMut, Env, Response, MessageInfo, StdResult};
/// # struct InstantiateMsg {
/// #     pub verifier: String,
/// #     pub beneficiary: String,
/// # }
/// # pub fn instantiate(
/// #     _deps: DepsMut,
/// #     _env: Env,
/// #     _info: MessageInfo,
/// #     _msg: InstantiateMsg,
/// # ) -> StdResult<Response> {
/// #     Ok(Response::new().add_attribute("action", "instantiate"))
/// # }
/// use cosmwasm_std::coins;
/// use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
///
/// let mut deps = mock_dependencies();
///
/// // Create some Addr instances for testing
/// let creator = deps.api.addr_make("creator");
/// let verifier = deps.api.addr_make("verifies");
/// let beneficiary = deps.api.addr_make("benefits");
///
/// let msg = InstantiateMsg {
///     verifier: verifier.to_string(),
///     beneficiary: beneficiary.to_string(),
/// };
/// let info = message_info(&creator, &coins(1000, "earth"));
/// let response = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
/// assert_eq!(response.messages.len(), 0);
/// ```
pub fn message_info(sender: &Addr, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: sender.clone(),
        funds: funds.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_core::Uint128;

    use crate::coins;

    use super::*;

    #[test]
    fn message_info_works() {
        let addr = Addr::unchecked("cosmwasm1...");

        let info = message_info(&addr, &[]);
        assert_eq!(
            info,
            MessageInfo {
                sender: addr.clone(),
                funds: vec![],
            }
        );

        let info = message_info(&addr, &coins(123, "foo"));
        assert_eq!(
            info,
            MessageInfo {
                sender: addr.clone(),
                funds: vec![Coin {
                    amount: Uint128::new(123),
                    denom: "foo".to_string(),
                }],
            }
        );
    }
}
