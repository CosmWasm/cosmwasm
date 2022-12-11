use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::{Coin, Decimal, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// name of the derivative token (FIXME: auto-generate?)
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    /// TODO: does this make sense? Do we need to normalize on this?
    /// We don't even know the decimals of the native token
    pub decimals: u8,

    /// This is the validator that all tokens will be bonded to
    pub validator: String,

    /// this is how much the owner takes as a cut when someone unbonds
    /// TODO
    pub exit_tax: Decimal,
    /// This is the minimum amount we will pull out to reinvest, as well as a minumum
    /// that can be unbonded (to avoid needless staking tx)
    pub min_withdrawal: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Transfer moves the derivative token
    Transfer { recipient: String, amount: Uint128 },
    /// Bond will bond all staking tokens sent with the message and release derivative tokens
    Bond {},
    /// Unbond will "burn" the given amount of derivative tokens and send the unbonded
    /// staking tokens to the message sender (after exit tax is deducted)
    Unbond { amount: Uint128 },
    /// Claim is used to claim your native tokens that you previously "unbonded"
    /// after the chain-defined waiting period (eg. 3 weeks)
    Claim {},
    /// Reinvest will check for all accumulated rewards, withdraw them, and
    /// re-bond them to the same validator. Anyone can call this, which updates
    /// the value of the token (how much under custody).
    Reinvest {},
    /// _BondAllTokens can only be called by the contract itself, after all rewards have been
    /// withdrawn. This is an example of using "callbacks" in message flows.
    /// This can only be invoked by the contract itself as a return from Reinvest
    _BondAllTokens {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Balance shows the number of staking derivatives
    #[returns(BalanceResponse)]
    Balance { address: String },
    /// Claims shows the number of tokens this address can access when they are done unbonding
    #[returns(ClaimsResponse)]
    Claims { address: String },
    /// TokenInfo shows the metadata of the token for UIs
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Investment shows info on total staking tokens under custody,
    /// with which validator, as well as how many derivative tokens are lists.
    /// It also shows with the exit tax.
    #[returns(InvestmentResponse)]
    Investment {},
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct ClaimsResponse {
    pub claims: Uint128,
}

/// TokenInfoResponse is info to display the derivative token in a UI
#[cw_serde]
pub struct TokenInfoResponse {
    /// name of the derivative token
    pub name: String,
    /// symbol / ticker of the derivative token
    pub symbol: String,
    /// decimal places of the derivative token (for UI)
    pub decimals: u8,
}

#[cw_serde]
pub struct InvestmentResponse {
    pub token_supply: Uint128,
    pub staked_tokens: Coin,
    // ratio of staked_tokens / token_supply (or how many native tokens that one derivative token is nominally worth)
    pub nominal_value: Decimal,

    /// owner created the contract and takes a cut
    pub owner: String,
    /// this is how much the owner takes as a cut when someone unbonds
    pub exit_tax: Decimal,
    /// All tokens are bonded to this validator
    pub validator: String,
    /// This is the minimum amount we will pull out to reinvest, as well as a minumum
    /// that can be unbonded (to avoid needless staking tx)
    pub min_withdrawal: Uint128,
}
