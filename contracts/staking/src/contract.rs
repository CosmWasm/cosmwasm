use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Coin, Decimal, Decimal256, Deps, DepsMut,
    DistributionMsg, Env, MessageInfo, QuerierWrapper, QueryResponse, Response, StakingMsg,
    StdError, StdResult, Uint128, Uint256, WasmMsg,
};

use crate::errors::{StakingError, Unauthorized};
use crate::msg::{
    BalanceResponse, ClaimsResponse, ExecuteMsg, InstantiateMsg, InvestmentResponse, QueryMsg,
    TokenInfoResponse,
};
use crate::state::{
    load_item, may_load_map, save_item, save_map, update_item, InvestmentInfo, Supply, TokenInfo,
    KEY_INVESTMENT, KEY_TOKEN_INFO, KEY_TOTAL_SUPPLY, PREFIX_BALANCE, PREFIX_CLAIMS,
};

const FALLBACK_RATIO: Decimal = Decimal::one();

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // ensure the validator is registered
    let validator = deps.querier.query_validator(msg.validator.clone())?;
    if validator.is_none() {
        return Err(StdError::generic_err(format!(
            "{} is not in the current validator set",
            msg.validator
        )));
    }

    let token = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    };
    save_item(deps.storage, KEY_TOKEN_INFO, &token)?;

    let denom = deps.querier.query_bonded_denom()?;
    let invest = InvestmentInfo {
        owner: info.sender,
        exit_tax: msg.exit_tax,
        bond_denom: denom,
        validator: msg.validator,
        min_withdrawal: msg.min_withdrawal,
    };
    save_item(deps.storage, KEY_INVESTMENT, &invest)?;

    // set supply to 0
    let supply = Supply::default();
    save_item(deps.storage, KEY_TOTAL_SUPPLY, &supply)?;

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
    recipient: String,
    send: Uint128,
) -> StdResult<Response> {
    let rcpt_raw = deps.api.addr_canonicalize(&recipient)?;
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    let balance: Uint128 =
        may_load_map(deps.storage, PREFIX_BALANCE, &sender_raw)?.unwrap_or_default();
    save_map(
        deps.storage,
        PREFIX_BALANCE,
        &sender_raw,
        balance.checked_sub(send)?,
    )?;
    let balance: Uint128 =
        may_load_map(deps.storage, PREFIX_BALANCE, &rcpt_raw)?.unwrap_or_default();
    save_map(deps.storage, PREFIX_BALANCE, &rcpt_raw, balance + send)?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", send.to_string());
    Ok(res)
}

// get_bonded returns the total amount of delegations from contract
// it ensures they are all the same denom
fn get_bonded(querier: &QuerierWrapper, contract_addr: impl Into<String>) -> StdResult<Uint256> {
    let bonds = querier.query_all_delegations(contract_addr)?;
    if bonds.is_empty() {
        return Ok(Uint256::zero());
    }
    let denom = bonds[0].amount.denom.as_str();
    bonds.iter().try_fold(Uint256::zero(), |acc, d| {
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

fn assert_bonds(supply: &Supply, bonded: Uint256) -> StdResult<()> {
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
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    // ensure we have the proper denom
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    // payment finds the proper coin (or throws an error)
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == invest.bond_denom)
        .ok_or_else(|| StdError::generic_err(format!("No {} tokens sent", &invest.bond_denom)))?;

    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, env.contract.address)?;

    // calculate to_mint and update total supply
    let mut supply: Supply = load_item(deps.storage, KEY_TOTAL_SUPPLY)?;
    // TODO: this is just temporary check - we should use dynamic query or have a way to recover
    assert_bonds(&supply, bonded)?;
    // note that the conversion to Uint128 limits payment amounts to `u128::MAX`
    let to_mint = if supply.issued.is_zero() || bonded.is_zero() {
        Uint128::try_from(payment.amount.mul_floor(FALLBACK_RATIO))?
    } else {
        Uint128::try_from(payment.amount.multiply_ratio(supply.issued, bonded))?
    };
    supply.bonded = bonded + payment.amount;
    supply.issued += to_mint;
    save_item(deps.storage, KEY_TOTAL_SUPPLY, &supply)?;

    // update the balance of the sender
    let balance: Uint128 =
        may_load_map(deps.storage, PREFIX_BALANCE, &sender_raw)?.unwrap_or_default();
    save_map(deps.storage, PREFIX_BALANCE, &sender_raw, balance + to_mint)?;

    // bond them to the validator
    let res = Response::new()
        .add_attribute("action", "bond")
        .add_attribute("from", info.sender)
        .add_attribute("bonded", payment.amount)
        .add_attribute("minted", to_mint)
        .add_message(StakingMsg::Delegate {
            validator: invest.validator,
            amount: payment.clone(),
        });
    Ok(res)
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    // ensure it is big enough to care
    if amount < invest.min_withdrawal {
        return Err(StdError::generic_err(format!(
            "Must unbond at least {} {}",
            invest.min_withdrawal, invest.bond_denom
        )));
    }

    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let owner_raw = deps.api.addr_canonicalize(invest.owner.as_str())?;

    // calculate tax and remainder to unbond
    let tax = amount.mul_floor(invest.exit_tax);

    // deduct all from the account
    let balance: Uint128 =
        may_load_map(deps.storage, PREFIX_BALANCE, &sender_raw)?.unwrap_or_default();
    save_map(
        deps.storage,
        PREFIX_BALANCE,
        &sender_raw,
        balance.checked_sub(amount)?,
    )?;
    if tax > Uint128::new(0) {
        // add tax to the owner
        let balance: Uint128 =
            may_load_map(deps.storage, PREFIX_BALANCE, &owner_raw)?.unwrap_or_default();
        save_map(deps.storage, PREFIX_BALANCE, &owner_raw, balance + tax)?;
    }

    // re-calculate bonded to ensure we have real values
    // bonded is the total number of tokens we have delegated from this address
    let bonded = get_bonded(&deps.querier, env.contract.address)?;

    // calculate how many native tokens this is worth and update supply
    let remainder = amount.checked_sub(tax)?;
    let mut supply: Supply = load_item(deps.storage, KEY_TOTAL_SUPPLY)?;
    // TODO: this is just temporary check - we should use dynamic query or have a way to recover
    assert_bonds(&supply, bonded)?;
    let unbond = Uint256::from(remainder).multiply_ratio(bonded, supply.issued);
    supply.bonded = bonded.checked_sub(unbond)?;
    supply.issued = supply.issued.checked_sub(remainder)?;
    supply.claims += unbond;
    save_item(deps.storage, KEY_TOTAL_SUPPLY, &supply)?;

    // add a claim to this user to get their tokens after the unbonding period
    let claim: Uint256 =
        may_load_map(deps.storage, PREFIX_CLAIMS, &sender_raw)?.unwrap_or_default();
    save_map(deps.storage, PREFIX_CLAIMS, &sender_raw, claim + unbond)?;

    // unbond them
    let res = Response::new()
        .add_attribute("action", "unbond")
        .add_attribute("to", info.sender)
        .add_attribute("unbonded", unbond)
        .add_attribute("burnt", amount)
        .add_message(StakingMsg::Undelegate {
            validator: invest.validator,
            amount: Coin::new(unbond, &invest.bond_denom),
        });
    Ok(res)
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    // find how many tokens the contract has
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    let mut balance = deps
        .querier
        .query_balance(env.contract.address, invest.bond_denom)?;
    if balance.amount < invest.min_withdrawal.into() {
        return Err(StdError::generic_err(
            "Insufficient balance in contract to process claim",
        ));
    }

    // check how much to send - min(balance, claims[sender]), and reduce the claim
    let sender_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let claim = may_load_map(deps.storage, PREFIX_CLAIMS, &sender_raw)?
        .ok_or_else(|| StdError::generic_err("no claim for this address"))?;
    let to_send = balance.amount.min(claim);
    save_map(
        deps.storage,
        PREFIX_CLAIMS,
        &sender_raw,
        claim.checked_sub(to_send)?,
    )?;

    // update total supply (lower claim)
    let mut supply: Supply = load_item(deps.storage, KEY_TOTAL_SUPPLY)?;
    supply.claims = supply.claims.checked_sub(to_send)?;
    save_item(deps.storage, KEY_TOTAL_SUPPLY, &supply)?;

    // transfer tokens to the sender
    balance.amount = to_send;
    let res = Response::new()
        .add_attribute("action", "claim")
        .add_attribute("from", &info.sender)
        .add_attribute("amount", to_send)
        .add_message(BankMsg::Send {
            to_address: info.sender.into(),
            amount: vec![balance],
        });
    Ok(res)
}

/// reinvest will withdraw all pending rewards,
/// then issue a callback to itself via _bond_all_tokens
/// to reinvest the new earnings (and anything else that accumulated)
pub fn reinvest(deps: DepsMut, env: Env, _info: MessageInfo) -> StdResult<Response> {
    let contract_addr = env.contract.address;
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    let msg = to_json_binary(&ExecuteMsg::_BondAllTokens {})?;

    // and bond them to the validator
    let res = Response::new()
        .add_message(DistributionMsg::WithdrawDelegatorReward {
            validator: invest.validator,
        })
        .add_message(WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        });
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
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    let mut balance = deps
        .querier
        .query_balance(env.contract.address, &invest.bond_denom)?;

    // we deduct pending claims from our account balance before reinvesting.
    // if there is not enough funds, we just return a no-op
    let updated = update_item(deps.storage, KEY_TOTAL_SUPPLY, |mut supply: Supply| {
        balance.amount = balance.amount.checked_sub(supply.claims)?;
        // this just triggers the "no op" case if we don't have min_withdrawal left to reinvest
        balance.amount.checked_sub(invest.min_withdrawal.into())?;
        supply.bonded += balance.amount;
        Ok(supply)
    });
    match updated {
        Ok(_) => {}
        // if it is below the minimum, we do a no-op (do not revert other state from withdrawal)
        Err(StdError::Overflow { .. }) => return Ok(Response::default()),
        Err(e) => return Err(e.into()),
    }

    // and bond them to the validator
    let res = Response::new()
        .add_attribute("action", "reinvest")
        .add_attribute("bonded", balance.amount)
        .add_message(StakingMsg::Delegate {
            validator: invest.validator,
            amount: balance,
        });
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Investment {} => to_json_binary(&query_investment(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, &address)?),
        QueryMsg::Claims { address } => to_json_binary(&query_claims(deps, &address)?),
    }
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let TokenInfo {
        name,
        symbol,
        decimals,
    } = load_item(deps.storage, KEY_TOKEN_INFO)?;

    Ok(TokenInfoResponse {
        name,
        symbol,
        decimals,
    })
}

pub fn query_balance(deps: Deps, address: &str) -> StdResult<BalanceResponse> {
    let address_raw = deps.api.addr_canonicalize(address)?;
    let balance = may_load_map(deps.storage, PREFIX_BALANCE, &address_raw)?.unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_claims(deps: Deps, address: &str) -> StdResult<ClaimsResponse> {
    let address_raw = deps.api.addr_canonicalize(address)?;
    let claims = may_load_map(deps.storage, PREFIX_CLAIMS, &address_raw)?.unwrap_or_default();
    Ok(ClaimsResponse { claims })
}

pub fn query_investment(deps: Deps) -> StdResult<InvestmentResponse> {
    let invest: InvestmentInfo = load_item(deps.storage, KEY_INVESTMENT)?;
    let supply: Supply = load_item(deps.storage, KEY_TOTAL_SUPPLY)?;

    let res = InvestmentResponse {
        owner: invest.owner.into(),
        exit_tax: invest.exit_tax,
        validator: invest.validator,
        min_withdrawal: invest.min_withdrawal,
        token_supply: supply.issued,
        staked_tokens: Coin::new(supply.bonded, invest.bond_denom),
        nominal_value: if supply.issued.is_zero() {
            FALLBACK_RATIO
        } else {
            // TODO: use Decimal256???
            Decimal256::from_ratio(supply.bonded, supply.issued)
                .try_into()
                .map_err(|_| StdError::generic_err("nominal value too high"))?
        },
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockQuerier, StakingQuerier, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{coin, coins, Addr, Coin, CosmosMsg, Decimal, FullDelegation, Validator};
    use std::str::FromStr;

    fn sample_validator(addr: &str) -> Validator {
        Validator::create(
            addr.to_owned(),
            Decimal::percent(3),
            Decimal::percent(10),
            Decimal::percent(1),
        )
    }

    fn sample_delegation(validator_addr: &str, amount: Coin) -> FullDelegation {
        FullDelegation::create(
            Addr::unchecked(MOCK_CONTRACT_ADDR),
            validator_addr.to_owned(),
            amount.clone(),
            amount,
            vec![],
        )
    }

    fn set_validator(querier: &mut MockQuerier) {
        querier.staking =
            StakingQuerier::new("ustake", &[sample_validator(DEFAULT_VALIDATOR)], &[]);
    }

    fn set_delegation(querier: &mut MockQuerier, amount: u128, denom: &str) {
        querier.staking.update(
            "ustake",
            &[sample_validator(DEFAULT_VALIDATOR)],
            &[sample_delegation(DEFAULT_VALIDATOR, coin(amount, denom))],
        );
    }

    const DEFAULT_VALIDATOR: &str = "default-validator";

    fn default_init(tax_percent: u64, min_withdrawal: u128) -> InstantiateMsg {
        InstantiateMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 9,
            validator: String::from(DEFAULT_VALIDATOR),
            exit_tax: Decimal::percent(tax_percent),
            min_withdrawal: Uint128::new(min_withdrawal),
        }
    }

    fn get_balance(deps: Deps, addr: &Addr) -> Uint128 {
        query_balance(deps, addr.as_str()).unwrap().balance
    }

    fn get_claims(deps: Deps, addr: &Addr) -> Uint128 {
        query_claims(deps, addr.as_str()).unwrap().claims
    }

    #[test]
    fn initialization_with_missing_validator() {
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");

        deps.querier
            .staking
            .update("ustake", &[sample_validator("john")], &[]);

        let msg = InstantiateMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 9,
            validator: String::from("my-validator"),
            exit_tax: Decimal::percent(2),
            min_withdrawal: Uint128::new(50),
        };
        let info = message_info(&creator, &[]);

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
        let mut deps = mock_dependencies();
        let creator = deps.api.addr_make("creator");

        deps.querier.staking.update(
            "ustake",
            &[
                sample_validator("john"),
                sample_validator("mary"),
                sample_validator("my-validator"),
            ],
            &[],
        );

        let msg = InstantiateMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 0,
            validator: String::from("my-validator"),
            exit_tax: Decimal::percent(2),
            min_withdrawal: Uint128::new(50),
        };
        let info = message_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // token info is proper
        let token = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(&token.name, &msg.name);
        assert_eq!(&token.symbol, &msg.symbol);
        assert_eq!(token.decimals, msg.decimals);

        // no balance
        assert_eq!(get_balance(deps.as_ref(), &creator), Uint128::new(0));
        // no claims
        assert_eq!(get_claims(deps.as_ref(), &creator), Uint128::new(0));

        // investment info correct
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(&invest.owner, creator.as_str());
        assert_eq!(&invest.validator, &msg.validator);
        assert_eq!(invest.exit_tax, msg.exit_tax);
        assert_eq!(invest.min_withdrawal, msg.min_withdrawal);

        assert_eq!(invest.token_supply, Uint128::new(0));
        assert_eq!(invest.staked_tokens, coin(0, "ustake"));
        assert_eq!(invest.nominal_value, Decimal::one());
    }

    #[test]
    fn bonding_issues_tokens() {
        let mut deps = mock_dependencies();
        set_validator(&mut deps.querier);

        let creator = deps.api.addr_make("creator");
        let bob = deps.api.addr_make("bob");

        let instantiate_msg = default_init(2, 50);
        let info = message_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bond_msg = ExecuteMsg::Bond {};
        let info = message_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);

        // try to bond and make sure we trigger delegation
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let delegate = &res.messages[0].msg;
        match delegate {
            CosmosMsg::Staking(StakingMsg::Delegate { validator, amount }) => {
                assert_eq!(validator.as_str(), DEFAULT_VALIDATOR);
                assert_eq!(amount, &coin(1000, "ustake"));
            }
            _ => panic!("Unexpected message: {delegate:?}"),
        }

        // bob got 1000 DRV for 1000 stake at a 1.0 ratio
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(1000));

        // investment info correct (updated supply)
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128::new(1000));
        assert_eq!(invest.staked_tokens, coin(1000, "ustake"));
        assert_eq!(invest.nominal_value, Decimal::one());
    }

    #[test]
    fn rebonding_changes_pricing() {
        let mut deps = mock_dependencies();
        set_validator(&mut deps.querier);

        let creator = deps.api.addr_make("creator");
        let bob = deps.api.addr_make("bob");
        let alice = deps.api.addr_make("alice");
        let contract = deps.api.addr_make(MOCK_CONTRACT_ADDR);

        let instantiate_msg = default_init(2, 50);
        let info = message_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bond_msg = ExecuteMsg::Bond {};
        let info = message_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1000, "ustake");

        // fake a reinvestment (this must be sent by the contract itself)
        let rebond_msg = ExecuteMsg::_BondAllTokens {};
        let info = message_info(&contract, &[]);
        deps.querier
            .bank
            .update_balance(&contract, coins(500, "ustake"));
        let mut env = mock_env();
        env.contract.address = contract.clone();
        let _ = execute(deps.as_mut(), env, info, rebond_msg).unwrap();

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1500, "ustake");

        // we should now see 1000 issues and 1500 bonded (and a price of 1.5)
        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128::new(1000));
        assert_eq!(invest.staked_tokens, coin(1500, "ustake"));
        let ratio = Decimal::from_str("1.5").unwrap();
        assert_eq!(invest.nominal_value, ratio);

        // we bond some other tokens and get a different issuance price (maintaining the ratio)
        let bond_msg = ExecuteMsg::Bond {};
        let info = message_info(&alice, &[coin(3000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 3000, "ustake");

        // alice should have gotten 2000 DRV for the 3000 stake, keeping the ratio at 1.5
        assert_eq!(get_balance(deps.as_ref(), &alice), Uint128::new(2000));

        let invest = query_investment(deps.as_ref()).unwrap();
        assert_eq!(invest.token_supply, Uint128::new(3000));
        assert_eq!(invest.staked_tokens, coin(4500, "ustake"));
        assert_eq!(invest.nominal_value, ratio);
    }

    #[test]
    fn bonding_fails_with_wrong_denom() {
        let mut deps = mock_dependencies();
        set_validator(&mut deps.querier);

        let creator = deps.api.addr_make("creator");
        let bob = deps.api.addr_make("bob");

        let instantiate_msg = default_init(2, 50);
        let info = message_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bond_msg = ExecuteMsg::Bond {};
        let info = message_info(&bob, &[coin(500, "photon")]);

        // try to bond and make sure we trigger delegation
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg);
        match res.unwrap_err() {
            StakingError::Std {
                original: StdError::GenericErr { msg, .. },
            } => assert_eq!(msg, "No ustake tokens sent"),
            err => panic!("Unexpected error: {err:?}"),
        };
    }

    #[test]
    fn unbonding_maintains_price_ratio() {
        let mut deps = mock_dependencies();
        set_validator(&mut deps.querier);

        let creator = deps.api.addr_make("creator");
        let bob = deps.api.addr_make("bob");
        let contract = deps.api.addr_make(MOCK_CONTRACT_ADDR);

        let instantiate_msg = default_init(10, 50);
        let info = message_info(&creator, &[]);

        // make sure we can instantiate with this
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // let's bond some tokens now
        let bond_msg = ExecuteMsg::Bond {};
        let info = message_info(&bob, &[coin(10, "random"), coin(1000, "ustake")]);
        let res = execute(deps.as_mut(), mock_env(), info, bond_msg).unwrap();
        assert_eq!(1, res.messages.len());

        // update the querier with new bond
        set_delegation(&mut deps.querier, 1000, "ustake");

        // fake a reinvestment (this must be sent by the contract itself)
        // after this, we see 1000 issues and 1500 bonded (and a price of 1.5)
        let rebond_msg = ExecuteMsg::_BondAllTokens {};
        let info = message_info(&contract, &[]);
        deps.querier
            .bank
            .update_balance(&contract, coins(500, "ustake"));
        let mut env = mock_env();
        env.contract.address = contract.clone();
        let _ = execute(deps.as_mut(), env, info, rebond_msg).unwrap();

        // update the querier with new bond, lower balance
        set_delegation(&mut deps.querier, 1500, "ustake");
        deps.querier.bank.update_balance(&contract, vec![]);

        // creator now tries to unbond these tokens - this must fail
        let unbond_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(600),
        };
        let info = message_info(&creator, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, unbond_msg);
        match res.unwrap_err() {
            StakingError::Std {
                original: StdError::Overflow { .. },
            } => {}
            err => panic!("Unexpected error: {err:?}"),
        }

        // bob unbonds 600 tokens at 10% tax...
        // 60 are taken and send to the owner
        // 540 are unbonded in exchange for 540 * 1.5 = 810 native tokens
        let unbond_msg = ExecuteMsg::Unbond {
            amount: Uint128::new(600),
        };
        let owner_cut = Uint128::new(60);
        let bobs_claim = Uint128::new(810);
        let bobs_balance = Uint128::new(400);
        let info = message_info(&bob, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, unbond_msg).unwrap();
        assert_eq!(1, res.messages.len());
        let delegate = &res.messages[0].msg;
        match delegate {
            CosmosMsg::Staking(StakingMsg::Undelegate { validator, amount }) => {
                assert_eq!(validator.as_str(), DEFAULT_VALIDATOR);
                assert_eq!(amount, &coin(bobs_claim.u128(), "ustake"));
            }
            _ => panic!("Unexpected message: {delegate:?}"),
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
