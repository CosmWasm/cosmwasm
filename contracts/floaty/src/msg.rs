use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

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
    /// Recurse will execute a query into itself up to depth-times and return
    /// Each step of the recursion may perform some extra work to test gas metering
    /// (`work` rounds of sha256 on contract).
    /// Now that we have Env, we can auto-calculate the address to recurse into
    #[returns(RecurseResponse)]
    Recurse { depth: u32, work: u32 },
}

#[cw_serde]
pub struct VerifierResponse {
    pub verifier: String,
}

#[cw_serde]
pub struct RecurseResponse {
    /// hashed is the result of running sha256 "work+1" times on the contract's human address
    pub hashed: Binary,
}
