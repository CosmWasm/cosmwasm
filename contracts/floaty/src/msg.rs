use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Coin};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub verifier: String,
    pub beneficiary: String,
}

/// MigrateMsg allows a privileged contract administrator to run
/// a migration on the contract. In this (demo) case it is just migrating
/// from one hackatom code to the same code, but taking advantage of the
/// migration step to set a new validator.
///
/// Note that the contract doesn't enforce permissions here, this is done
/// by blockchain logic (in the future by blockchain governance)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub verifier: String,
}

/// SudoMsg is only exposed for internal Cosmos SDK modules to call.
/// This is showing how we can expose "admin" functionality than can not be called by
/// external users or contracts, but only trusted (native/Go) code in the blockchain
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    StealFunds {
        recipient: String,
        amount: Vec<Coin>,
    },
}

// failure modes to help test wasmd, based on this comment
// https://github.com/cosmwasm/wasmd/issues/8#issuecomment-576146751
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Releasing all funds in the contract to the beneficiary. This is the only "proper" action of this demo contract.
    Release {},
    /// Infinite loop to burn cpu cycles (only run when metering is enabled)
    CpuLoop {},
    /// Infinite loop making storage calls (to test when their limit hits)
    StorageLoop {},
    /// Infinite loop reading and writing memory
    MemoryLoop {},
    /// Allocate large amounts of memory without consuming much gas
    AllocateLargeMemory { pages: u32 },
    /// Trigger a panic to ensure framework handles gracefully
    Panic {},
    /// Starting with CosmWasm 0.10, some API calls return user errors back to the contract.
    /// This triggers such user errors, ensuring the transaction does not fail in the backend.
    UserErrorsInApiCalls {},
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
