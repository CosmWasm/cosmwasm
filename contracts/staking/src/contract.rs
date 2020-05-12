use cosmwasm_std::{
    coin, generic_err, log, to_binary, unauthorized, Api, Binary, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier, StakingMsg, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::msg::{
    BalanceResponse, HandleMsg, InitMsg, InvestmentResponse, QueryMsg, TokenInfoResponse,
};
use crate::state::{
    balances, balances_read, claims, invest_info, invest_info_read, token_info, token_info_read,
    total_supply, total_supply_read, InvestmentInfo, Supply,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let token = TokenInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    };
    token_info(&mut deps.storage).save(&token)?;

    let denom = deps.querier.query_bonded_denom()?;
    let invest = InvestmentInfo {
        owner: env.message.sender,
        exit_tax: msg.exit_tax,
        bond_denom: denom,
        validator: msg.validator,
        min_withdrawl: msg.min_withdrawl,
    };
    invest_info(&mut deps.storage).save(&invest)?;

    // set supply to 0
    let supply = Supply::default();
    total_supply(&mut deps.storage).save(&supply)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Transfer { recipient, amount } => transfer(deps, env, recipient, amount),
        HandleMsg::Bond {} => bond(deps, env),
        HandleMsg::Unbond { amount } => unbond(deps, env, amount),
        HandleMsg::Reinvest {} => reinvest(deps, env),
        HandleMsg::_BondAllTokens {} => _bond_all_tokens(deps, env),
    }
}

pub fn transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    send: Uint128,
) -> StdResult<HandleResponse> {
    let rcpt_raw = deps.api.canonical_address(&recipient)?;
    let sender_raw = env.message.sender;

    let mut accounts = balances(&mut deps.storage);
    accounts.update(sender_raw.as_slice(), &mut |balance: Option<Uint128>| {
        balance.unwrap_or_default() - send
    })?;
    accounts.update(rcpt_raw.as_slice(), &mut |balance: Option<Uint128>| {
        Ok(balance.unwrap_or_default() + send)
    })?;

    let res = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "transfer"),
            log("from", deps.api.human_address(&sender_raw)?.as_str()),
            log("to", recipient.as_str()),
            log("amount", &send.to_string()),
        ],
        data: None,
    };
    Ok(res)
}

pub fn bond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_raw = env.message.sender;

    // ensure we have the proper denom
    let invest = invest_info_read(&deps.storage).load()?;
    // payment finds the proper coin (or throws an error)
    let payment = env
        .message
        .sent_funds
        .iter()
        .find(|x| x.denom == invest.bond_denom)
        .ok_or_else(|| generic_err(format!("No {} tokens sent", &invest.bond_denom)))?;

    // update total supply
    let mut to_mint = Uint128(0);
    let _ = total_supply(&mut deps.storage).update(&mut |mut supply| {
        to_mint = payment.amount.multiply_ratio(supply.issued, supply.bonded);
        supply.bonded += payment.amount;
        supply.issued += to_mint;
        Ok(supply)
    })?;

    // update the balance of the sender
    balances(&mut deps.storage).update(sender_raw.as_slice(), &mut |balance| {
        Ok(balance.unwrap_or_default() + to_mint)
    })?;

    // bond them to the validator
    let res = HandleResponse {
        messages: vec![StakingMsg::Delegate {
            validator: invest.validator,
            amount: payment.clone(),
        }
        .into()],
        log: vec![
            log("action", "bond"),
            log("from", deps.api.human_address(&sender_raw)?.as_str()),
            log("bonded", &payment.amount.to_string()),
            log("minted", &to_mint.to_string()),
        ],
        data: None,
    };
    Ok(res)
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender_raw = env.message.sender;

    let invest = invest_info_read(&deps.storage).load()?;
    // ensure it is big enough to care
    if amount < invest.min_withdrawl {
        return Err(generic_err(format!(
            "Must unbond at least {} {}",
            invest.min_withdrawl, invest.bond_denom
        )));
    }
    // calculate tax and remainer to unbond
    let tax = amount * invest.exit_tax;

    // deduct all from the account
    let mut accounts = balances(&mut deps.storage);
    accounts.update(sender_raw.as_slice(), &mut |balance| {
        balance.unwrap_or_default() - amount
    })?;
    if tax > Uint128(0) {
        // add tax to the owner
        accounts.update(invest.owner.as_slice(), &mut |balance: Option<Uint128>| {
            Ok(balance.unwrap_or_default() + tax)
        })?;
    }

    // calculate how many native tokens this is worth and update supply
    let remainder = (amount - tax)?;
    let mut unbond = Uint128(0);
    total_supply(&mut deps.storage).update(&mut |mut supply| {
        unbond = remainder.multiply_ratio(supply.bonded, supply.issued);
        supply.bonded = (supply.bonded - unbond)?;
        supply.issued = (supply.bonded - remainder)?;
        supply.claims += unbond;
        Ok(supply)
    })?;

    // add a claim to this user to get their tokens after the unbonding period
    claims(&mut deps.storage).update(sender_raw.as_slice(), &mut |claim| {
        Ok(claim.unwrap_or_default() + unbond)
    })?;

    // unbond them
    let res = HandleResponse {
        messages: vec![StakingMsg::Undelegate {
            validator: invest.validator,
            amount: coin(unbond.u128(), &invest.bond_denom),
        }
        .into()],
        log: vec![
            log("action", "unbond"),
            log("to", deps.api.human_address(&sender_raw)?.as_str()),
            log("unbonded", &unbond.to_string()),
            log("burnt", &amount.to_string()),
        ],
        data: None,
    };
    Ok(res)
}

/// reinvest will withdraw all pending rewards,
/// then issue a callback to itself via _bond_all_tokens
/// to reinvest the new earnings (and anything else that accumulated)
pub fn reinvest<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let contract_addr = deps.api.human_address(&env.contract.address)?;
    let invest = invest_info_read(&deps.storage).load()?;
    let msg = to_binary(&HandleMsg::_BondAllTokens {})?;

    // and bond them to the validator
    let res = HandleResponse {
        messages: vec![
            StakingMsg::Withdraw {
                validator: invest.validator,
                recipient: Some(contract_addr.clone()),
            }
            .into(),
            WasmMsg::Execute {
                contract_addr,
                msg,
                send: vec![],
            }
            .into(),
        ],
        log: vec![],
        data: None,
    };
    Ok(res)
}

pub fn _bond_all_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // this is just meant as a call-back to ourself
    if env.message.sender != env.contract.address {
        return Err(unauthorized());
    }

    // find how many tokens we have to bond
    let contract_human = deps.api.human_address(&env.contract.address)?;
    let invest = invest_info_read(&deps.storage).load()?;
    let mut balance = deps
        .querier
        .query_balance(contract_human, &invest.bond_denom)?;

    // we deduct pending claims from our account balance before reinvesting.
    // if there is not enough funds, we just return a no-op
    match total_supply(&mut deps.storage).update(&mut |mut supply| {
        balance.amount = (balance.amount - supply.claims)?;
        // this just triggers the "no op" case if we don't have min_withdrawl left to reinvest
        (balance.amount - invest.min_withdrawl)?;
        supply.bonded += balance.amount;
        Ok(supply)
    }) {
        Ok(_) => {}
        // if it is below the minimum, we do a no-op (do not revert other state from withdrawl)
        Err(StdError::Underflow { .. }) => return Ok(HandleResponse::default()),
        Err(e) => return Err(e),
    }

    // and bond them to the validator
    let res = HandleResponse {
        messages: vec![StakingMsg::Delegate {
            validator: invest.validator,
            amount: balance.clone(),
        }
        .into()],
        log: vec![
            log("action", "reinvest"),
            log("bonded", &balance.amount.to_string()),
        ],
        data: None,
    };
    Ok(res)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo {} => query_token_info(deps),
        QueryMsg::Investment {} => panic!("investment"),
        QueryMsg::Balance { address } => query_balance(deps, address),
    }
}

pub fn query_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let info = token_info_read(&deps.storage).load()?;
    to_binary(&info)
}

pub fn query_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<Binary> {
    let address_raw = deps.api.canonical_address(&address)?;
    let balance = balances_read(&deps.storage).load(address_raw.as_slice())?;
    to_binary(&BalanceResponse { balance })
}

pub fn query_investment<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let invest = invest_info_read(&deps.storage).load()?;
    let supply = total_supply_read(&deps.storage).load()?;

    let res = InvestmentResponse {
        owner: deps.api.human_address(&invest.owner)?,
        exit_tax: invest.exit_tax,
        validator: invest.validator,
        min_withdrawl: invest.min_withdrawl,
        token_supply: supply.issued,
        staked_tokens: coin(supply.bonded.u128(), &invest.bond_denom),
        nominal_value: supply.bonded.calc_ratio(supply.issued),
    };
    to_binary(&res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
