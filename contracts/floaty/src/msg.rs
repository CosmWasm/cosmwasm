use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub verifier: String,
    pub beneficiary: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Releasing all funds in the contract to the beneficiary. This is the only "proper" action of this demo contract.
    Release {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// returns a human-readable representation of the verifier
    /// use to ensure query path works in integration tests
    #[returns(VerifierResponse)]
    Verifier {},
    /// This returns cosmwasm_std::AllBalanceResponse to demo use of the querier
    #[returns(cosmwasm_std::AllBalanceResponse)]
    OtherBalance { address: String },
}

#[cw_serde]
pub struct VerifierResponse {
    pub verifier: String,
}
