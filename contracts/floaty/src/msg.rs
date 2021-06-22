use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Binary;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub verifier: String,
    pub beneficiary: String,
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Releasing all funds in the contract to the beneficiary. This is the only "proper" action of this demo contract.
    Release {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// returns a human-readable representation of the verifier
    /// use to ensure query path works in integration tests
    Verifier {},
    /// This returns cosmwasm_std::AllBalanceResponse to demo use of the querier
    OtherBalance { address: String },
    /// Recurse will execute a query into itself up to depth-times and return
    /// Each step of the recursion may perform some extra work to test gas metering
    /// (`work` rounds of sha256 on contract).
    /// Now that we have Env, we can auto-calculate the address to recurse into
    Recurse { depth: u32, work: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VerifierResponse {
    pub verifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RecurseResponse {
    /// hashed is the result of running sha256 "work+1" times on the contract's human address
    pub hashed: Binary,
}
