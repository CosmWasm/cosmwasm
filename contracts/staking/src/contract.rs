use cosmwasm_std::{
    attr, coin, entry_point, to_binary, BankMsg, Decimal, Deps, DepsMut, Env, HumanAddr,
    MessageInfo, QuerierWrapper, QueryResponse, Response, StakingMsg, StdError, StdResult, Uint128,
    WasmMsg,
};

use crate::errors::{StakingError, Unauthorized};
use crate::msg::{
    BalanceResponse, ClaimsResponse, ExecuteMsg, InitMsg, InvestmentResponse, QueryMsg,
    TokenInfoResponse,
};
use crate::state::{
    balances, balances_read, claims, claims_read, invest_info, invest_info_read, token_info,
    token_info_read, total_supply, total_supply_read, InvestmentInfo, Supply,
};

const FALLBACK_RATIO: Decimal = Decimal::one();

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    // ensure the validator is registered
    let vals = deps.querier.query_validators()?;
    if !vals.iter().any(|v| v.address == msg.validator) {
        return Err(StdError::generic_err(format!(
            "{} is not in the current validator set",
            msg.validator
        )));
    }

    let token = TokenInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    };
    token_info(deps.storage).save(&token)?;

    let denom = deps.querier.query_bonded_denom()?;
    let invest = InvestmentInfo {
        owner: deps.api.canonical_address(&info.sender)?,
        exit_tax: msg.exit_tax,
        bond_denom: denom,
        validator: msg.validator,
        min_withdrawal: msg.min_withdrawal,
    };
    invest_info(deps.storage).save(&invest)?;

    // set supply to 0
    let supply = Supply::default();
    total_supply(deps.storage).save(&supply)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StakingError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Bond {} => Ok(bond(deps, env, info)?),
        ExecuteMsg::Unbond { amount } => Ok(unbond(deps, env, info, amount)?),
        ExecuteMsg::Claim {} => Ok(claim(deps, env, info)?),
        ExecuteMsg::Reinvest {} => Ok(reinvest(deps, env, info)?),
        ExecuteMsg::_BondAllTokens {} => _bond_all_tokens(deps, env, info),
    }
}

pub fn transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: HumanAddr,
    send: Uint128,
) -> StdResult<Response> {
    let rcpt_raw = deps.api.canonical_address(&recipient)?;
    let sender_raw = deps.api.canonical_address(&info.sender)?;

    let mut accounts = balances(deps.storage);
    accounts.update(&sender_raw, |balance: Option<Uint128>| {
        balance.unwrap_or_default() - send
    })?;
    accounts.update(&rcpt_raw, |balance: Option<Uint128>| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + send)
    })?;

    let res = Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![
            attr("action", "transfer"),
            attr("from", info.sender),
            attr("to", recipient),
            attr("amount", send),
        ],
        data: None,
    };
    Ok(res)
}

// get_bonded returns the total amount of delegations from contract
// it ensures they are all the same denom
fn get_bonded(querier: &QuerierWrapper, contract: &HumanAddr) -> StdResult<Uint128> {
    let bonds = querier.query_all_delegations(contract)?;
    if bonds.is_empty() {
        return Ok(Uint128(0));
    }
    let denom = bonds[0].amount.denom.as_str();
    bonds.iter().fold(Ok(Uint128(0)), |racc, d| {
        let acc = racc?;
        if d.amount.denom.as_str() != denom {
            Err(StdError::generic_err(format!(
                "different denoms in bonds: '{}' vs '{}'",
                denom, &d.amount.denom
            )))
        } else {
            Ok(acc + d.amount.amount)
        }
    })
}

fn assert_bonds(supply: &Supply, bonded: Uint128) -> StdResult<()> {
    if supply.bonded != bonded {
        Err(StdError::generic_err(format!(
            "Stored bonded {}, but query bonded: {}",
            supply.bonded, bonded
        )))
    } else {
        Ok(())
    }
}

pub fn bond(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender_raw = deps.api.canonical_address(&info.sender)?;

    // ensure we have the proper denom
    let invest = invest_info_read(deps.storage).load()?;
    // payment finds the proper coin (or throws an error)
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == invest.bond_denom)
        .ok_or_else(|| StdError::generic_err(format!("No {} tokens sent", &invest.bond_denom)))?;

    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, &env.contract.address)?;

    // calculate to_mint and update total supply
    let mut totals = total_supply(deps.storage);
    let mut supply = totals.load()?;
    // TODO: this is just temporary check - we should use dynamic query or have a way to recover
    assert_bonds(&supply, bonded)?;
    let to_mint = if supply.issued.is_zero() || bonded.is_zero() {
        FALLBACK_RATIO * payment.amount
    } else {
        payment.amount.multiply_ratio(supply.issued, bonded)
    };
    supply.bonded = bonded + payment.amount;
    supply.issued += to_mint;
    totals.save(&supply)?;

    // update the balance of the sender
    balances(deps.storage).update(&sender_raw, |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + to_mint)
    })?;

    // bond them to the validator
    let res = Response {
        submessages: vec![],
        messages: vec![StakingMsg::Delegate {
            validator: invest.validator,
            amount: payment.clone(),
        }
        .into()],
        attributes: vec![
            attr("action", "bond"),
            attr("from", info.sender),
            attr("bonded", payment.amount),
            attr("minted", to_mint),
        ],
        data: None,
    };
    Ok(res)
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let sender_raw = deps.api.canonical_address(&info.sender)?;

    let invest = invest_info_read(deps.storage).load()?;
    // ensure it is big enough to care
    if amount < invest.min_withdrawal {
        return Err(StdError::generic_err(format!(
            "Must unbond at least {} {}",
            invest.min_withdrawal, invest.bond_denom
        )));
    }
    // calculate tax and remainer to unbond
    let tax = amount * invest.exit_tax;

    // deduct all from the account
    let mut accounts = balances(deps.storage);
    accounts.update(&sender_raw, |balance| -> StdResult<_> {
        balance.unwrap_or_default() - amount
    })?;
    if tax > Uint128(0) {
        // add tax to the owner
        accounts.update(&invest.owner, |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + tax)
        })?;
    }

    // re-calculate bonded to ensure we have real values
    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, &env.contract.address)?;

    // calculate how many native tokens this is worth and update supply
    let remainder = (amount - tax)?;
    let mut totals = total_supply(deps.storage);
    let mut supply = totals.load()?;
    // TODO: this is just temporary check - we should use dynamic query or have a way to recover
    assert_bonds(&supply, bonded)?;
    let unbond = remainder.multiply_ratio(bonded, supply.issued);
    supply.bonded = (bonded - unbond)?;
    supply.issued = (supply.issued - remainder)?;
    supply.claims += unbond;
    totals.save(&supply)?;

    // add a claim to this user to get their tokens after the unbonding period
    claims(deps.storage).update(&sender_raw, |claim| -> StdResult<_> {
        Ok(claim.unwrap_or_default() + unbond)
    })?;

    // unbond them
    let res = Response {
        submessages: vec![],
        messages: vec![StakingMsg::Undelegate {
            validator: invest.validator,
            amount: coin(unbond.u128(), &invest.bond_denom),
        }
        .into()],
        attributes: vec![
            attr("action", "unbond"),
            attr("to", info.sender),
            attr("unbonded", unbond),
            attr("burnt", amount),
        ],
        data: None,
    };
    Ok(res)
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    // find how many tokens the contract has
    let invest = invest_info_read(deps.storage).load()?;
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &invest.bond_denom)?;
    if balance.amount < invest.min_withdrawal {
        return Err(StdError::generic_err(
            "Insufficient balance in contract to process claim",
        ));
    }

    // check how much to send - min(balance, claims[sender]), and reduce the claim
    let sender_raw = deps.api.canonical_address(&info.sender)?;
    let mut to_send = balance.amount;
    claims(deps.storage).update(sender_raw.as_slice(), |claim| {
        let claim = claim.ok_or_else(|| StdError::generic_err("no claim for this address"))?;
        to_send = to_send.min(claim);
        claim - to_send
    })?;

    // update total supply (lower claim)
    total_supply(deps.storage).update(|mut supply| -> StdResult<_> {
        supply.claims = (supply.claims - to_send)?;
        Ok(supply)
    })?;

    // transfer tokens to the sender
    balance.amount = to_send;
    let res = Response {
        submessages: vec![],
        messages: vec![BankMsg::Send {
            to_address: info.sender.clone(),
            amount: vec![balance],
        }
        .into()],
        attributes: vec![
            attr("action", "claim"),
            attr("from", info.sender),
            attr("amount", to_send),
        ],
        data: None,
    };
    Ok(res)
}

/// reinvest will withdraw all pending rewards,
/// then issue a callback to itself via _bond_all_tokens
/// to reinvest the new earnings (and anything else that accumulated)
pub fn reinvest(deps: DepsMut, env: Env, _info: MessageInfo) -> StdResult<Response> {
    let contract_addr = env.contract.address;
    let invest = invest_info_read(deps.storage).load()?;
    let msg = to_binary(&ExecuteMsg::_BondAllTokens {})?;

    // and bond them to the validator
    let res = Response {
        submessages: vec![],
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
        attributes: vec![],
        data: None,
    };
    Ok(res)
}

pub fn _bond_all_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, StakingError> {
    // this is just meant as a call-back to ourself
    if info.sender != env.contract.address {
        return Err(Unauthorized {}.build());
    }

    // find how many tokens we have to bond
    let invest = invest_info_read(deps.storage).load()?;
    let mut balance = deps
        .querier
        .query_balance(&env.contract.address, &invest.bond_denom)?;

    // we deduct pending claims from our account balance before reinvesting.
    // if there is not enough funds, we just return a no-op
    match total_supply(deps.storage).update(|mut supply| {
        balance.amount = (balance.amount - supply.claims)?;
        // this just triggers the "no op" case if we don't have min_withdrawal left to reinvest
        (balance.amount - invest.min_withdrawal)?;
        supply.bonded += balance.amount;
        Ok(supply)
    }) {
        Ok(_) => {}
        // if it is below the minimum, we do a no-op (do not revert other state from withdrawal)
        Err(StdError::Underflow { .. }) => return Ok(Response::default()),
        Err(e) => return Err(e.into()),
    }

    // and bond them to the validator
    let res = Response {
        submessages: vec![],
        messages: vec![StakingMsg::Delegate {
            validator: invest.validator,
            amount: balance.clone(),
        }
        .into()],
        attributes: vec![attr("action", "reinvest"), attr("bonded", balance.amount)],
        data: None,
    };
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Investment {} => to_binary(&query_investment(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::Claims { address } => to_binary(&query_claims(deps, address)?),
    }
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    token_info_read(deps.storage).load()
}

pub fn query_balance(deps: Deps, address: HumanAddr) -> StdResult<BalanceResponse> {
    let address_raw = deps.api.canonical_address(&address)?;
    let balance = balances_read(deps.storage)
        .may_load(address_raw.as_slice())?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_claims(deps: Deps, address: HumanAddr) -> StdResult<ClaimsResponse> {
    let address_raw = deps.api.canonical_address(&address)?;
    let claims = claims_read(deps.storage)
        .may_load(address_raw.as_slice())?
        .unwrap_or_default();
    Ok(ClaimsResponse { claims })
}

pub fn query_investment(deps: Deps) -> StdResult<InvestmentResponse> {
    let invest = invest_info_read(deps.storage).load()?;
    let supply = total_supply_read(deps.storage).load()?;

    let res = InvestmentResponse {
        owner: deps.api.human_address(&invest.owner)?,
        exit_tax: invest.exit_tax,
        validator: invest.validator,
        min_withdrawal: invest.min_withdrawal,
        token_supply: supply.issued,
        staked_tokens: coin(supply.bonded.u128(), &invest.bond_denom),
        nominal_value: if supply.issued.is_zero() {
            FALLBACK_RATIO
        } else {
            Decimal::from_ratio(supply.bonded, supply.issued)
        },
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockQuerier, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{coins, Coin, CosmosMsg, Decimal, FullDelegation, Validator};
    use std::str::FromStr;

    fn sample_validator<U: Into<HumanAddr>>(addr: U) -> Validator {
        Validator {
            address: addr.into(),
            commission: Decimal::percent(3),
            max_commission: Decimal::percent(10),
            max_change_rate: Decimal::percent(1),
        }
    }

    fn sample_delegation<U: Into<HumanAddr>>(addr: U, amount: Coin) -> FullDelegation {
        let can_redelegate = amount.clone();
        FullDelegation {
            validator: addr.into(),
            delegator: HumanAddr::from(MOCK_CONTRACT_ADDR),
            amount,
            can_redelegate,
            accumulated_rewards: Vec::new(),
        }
    }

    fn set_validator(querier: &mut MockQuerier) {
        querier.update_staking("ustake", &[sample_validator(DEFAULT_VALIDATOR)], &[]);
    }

    fn set_delegation(querier: &mut MockQuerier, amount: u128, denom: &str) {
        querier.update_staking(
            "ustake",
            &[sample_validator(DEFAULT_VALIDATOR)],
            &[sample_delegation(DEFAULT_VALIDATOR, coin(amount, denom))],
        );
    }

    const DEFAULT_VALIDATOR: &str = "default-validator";

    fn default_init(tax_percent: u64, min_withdrawal: u128) -> InitMsg {
        InitMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 9,
            validator: HumanAddr::from(DEFAULT_VALIDATOR),
            exit_tax: Decimal::percent(tax_percent),
            min_withdrawal: Uint128(min_withdrawal),
        }
    }

    fn get_balance<U: Into<HumanAddr>>(deps: Deps, addr: U) -> Uint128 {
        query_balance(deps, addr.into()).unwrap().balance
    }

    fn get_claims<U: Into<HumanAddr>>(deps: Deps, addr: U) -> Uint128 {
        query_claims(deps, addr.into()).unwrap().claims
    }

    #[test]
    fn initialization_with_missing_validator() {
        let mut deps = mock_dependencies(&[]);
        deps.querier
            .update_staking("ustake", &[sample_validator("john")], &[]);

        let creator = HumanAddr::from("creator");
        let msg = InitMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 9,
            validator: HumanAddr::from("my-validator"),
            exit_tax: Decimal::percent(2),
            min_withdrawal: Uint128(50),
        };
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "my-validator is not in the current validator set")
            }
            _ => panic!("expected unregistered validator error"),
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        deps.querier.update_staking(
            "ustake",
            &[
                sample_validator("john"),
                sample_validator("mary"),
                sample_validator("my-validator"),
            ],
            &[],
        );

        let creator = HumanAddr::from("creator");
        let msg = InitMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 0,
            validator: HumanAddr::from("my-validator"),
            exit_tax: Decimal::percent(2),
            min_withdrawal: Uint128(50),
        };
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // token info is proper
        let token = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(&token.name, &msg.name);
        assert_eq!(&token.symbol, &msg.symbol);
        assert_eq!(token.decimals, msg.decimals);

        // no balance
        assert_eq!(get_balance(deps.as_ref(), &creator), Uint128(0));
        // no claims
        assert_eq!(get_claims(deps.as_ref(), &creator), Uint128(0));

        // investment info correct
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(&invest.owner, &creator);
        assert_eq!(&invest.validator, &msg.validator);
        assert_eq!(invest.exit_tax, msg.exit_tax);
        assert_eq!(invest.min_withdrawal, msg.min_withdrawal);

        assert_eq!(invest.token_supply, Uint128(0));
        assert_eq!(invest.staked_tokens, coin(0, "ustake"));
        assert_eq!(invest.nominal_value, Decimal::one());
    }

    #[test]
    fn bonding_issues_tokens() {
        let mut deps = mock_dependencies(&[]);
        set_validator(&mut deps.querier);

        let creator = HumanAddr::from("creator");
        let init_msg = default_init(2, 50);
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bob = HumanAddr::from("bob");
        let bond_msg = ExecuteMsg::Bond {};
        let info = mock_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);

        // try to bond and make sure we trigger delegation
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let delegate = &res.messages[0];
        match delegate {
            CosmosMsg::Staking(StakingMsg::Delegate { validator, amount }) => {
                assert_eq!(validator.as_str(), DEFAULT_VALIDATOR);
                assert_eq!(amount, &coin(1000, "ustake"));
            }
            _ => panic!("Unexpected message: {:?}", delegate),
        }

        // bob got 1000 DRV for 1000 stake at a 1.0 ratio
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128(1000));

        // investment info correct (updated supply)
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128(1000));
        assert_eq!(invest.staked_tokens, coin(1000, "ustake"));
        assert_eq!(invest.nominal_value, Decimal::one());
    }

    #[test]
    fn rebonding_changes_pricing() {
        let mut deps = mock_dependencies(&[]);
        set_validator(&mut deps.querier);

        let creator = HumanAddr::from("creator");
        let init_msg = default_init(2, 50);
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bob = HumanAddr::from("bob");
        let bond_msg = ExecuteMsg::Bond {};
        let info = mock_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1000, "ustake");

        // fake a reinvestment (this must be sent by the contract itself)
        let rebond_msg = ExecuteMsg::_BondAllTokens {};
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "ustake"));
        let _ = execute(deps.as_mut(), mock_env(), info, rebond_msg).unwrap();

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1500, "ustake");

        // we should now see 1000 issues and 1500 bonded (and a price of 1.5)
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128(1000));
        assert_eq!(invest.staked_tokens, coin(1500, "ustake"));
        let ratio = Decimal::from_str("1.5").unwrap();
        assert_eq!(invest.nominal_value, ratio);

        // we bond some other tokens and get a different issuance price (maintaining the ratio)
        let alice = HumanAddr::from("alice");
        let bond_msg = ExecuteMsg::Bond {};
        let info = mock_info(&alice, &[coin(3000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 3000, "ustake");

        // alice should have gotten 2000 DRV for the 3000 stake, keeping the ratio at 1.5
        assert_eq!(get_balance(deps.as_ref(), &alice), Uint128(2000));

        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128(3000));
        assert_eq!(invest.staked_tokens, coin(4500, "ustake"));
        assert_eq!(invest.nominal_value, ratio);
    }

    #[test]
    fn bonding_fails_with_wrong_denom() {
        let mut deps = mock_dependencies(&[]);
        set_validator(&mut deps.querier);

        let creator = HumanAddr::from("creator");
        let init_msg = default_init(2, 50);
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bob = HumanAddr::from("bob");
        let bond_msg = ExecuteMsg::Bond {};
        let info = mock_info(&bob, &[coin(500, "photon")]);

        // try to bond and make sure we trigger delegation
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg);
        match res.unwrap_err() {
            StakingError::Std {
                original: StdError::GenericErr { msg, .. },
            } => assert_eq!(msg, "No ustake tokens sent"),
            err => panic!("Unexpected error: {:?}", err),
        };
    }

    #[test]
    fn unbonding_maintains_price_ratio() {
        let mut deps = mock_dependencies(&[]);
        set_validator(&mut deps.querier);

        let creator = HumanAddr::from("creator");
        let init_msg = default_init(10, 50);
        let info = mock_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bob = HumanAddr::from("bob");
        let bond_msg = ExecuteMsg::Bond {};
        let info = mock_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1000, "ustake");

        // fake a reinvestment (this must be sent by the contract itself)
        // after this, we see 1000 issues and 1500 bonded (and a price of 1.5)
        let rebond_msg = ExecuteMsg::_BondAllTokens {};
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "ustake"));
        let _ = execute(deps.as_mut(), mock_env(), info, rebond_msg).unwrap();

        // update the querier with new bond, lower balance
        set_delegation(&mut deps.querier, 1500, "ustake");
        deps.querier.update_balance(MOCK_CONTRACT_ADDR, vec![]);

        // creator now tries to unbond these tokens - this must fail
        let unbond_msg = ExecuteMsg::Unbond {
            amount: Uint128(600),
        };
        let info = mock_info(&creator, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, unbond_msg);
        match res.unwrap_err() {
            StakingError::Std {
                original: StdError::Underflow { .. },
            } => {}
            err => panic!("Unexpected error: {:?}", err),
        }

        // bob unbonds 600 tokens at 10% tax...
        // 60 are taken and send to the owner
        // 540 are unbonded in exchange for 540 * 1.5 = 810 native tokens
        let unbond_msg = ExecuteMsg::Unbond {
            amount: Uint128(600),
        };
        let owner_cut = Uint128(60);
        let bobs_claim = Uint128(810);
        let bobs_balance = Uint128(400);
        let info = mock_info(&bob, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, unbond_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let delegate = &res.messages[0];
        match delegate {
            CosmosMsg::Staking(StakingMsg::Undelegate { validator, amount }) => {
                assert_eq!(validator.as_str(), DEFAULT_VALIDATOR);
                assert_eq!(amount, &coin(bobs_claim.u128(), "ustake"));
            }
            _ => panic!("Unexpected message: {:?}", delegate),
        }

        // update the querier with new bond, lower balance
        set_delegation(&mut deps.querier, 690, "ustake");

        // check balances
        assert_eq!(get_balance(deps.as_ref(), &bob), bobs_balance);
        assert_eq!(get_balance(deps.as_ref(), &creator), owner_cut);
        // proper claims
        assert_eq!(get_claims(deps.as_ref(), &bob), bobs_claim);

        // supplies updated, ratio the same (1.5)
        let ratio = Decimal::from_str("1.5").unwrap();

        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, bobs_balance + owner_cut);
        assert_eq!(invest.staked_tokens, coin(690, "ustake")); // 1500 - 810
        assert_eq!(invest.nominal_value, ratio);
    }
}
