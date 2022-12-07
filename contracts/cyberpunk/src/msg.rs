use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum ExecuteMsg {
    /// Hashes some data. Uses CPU and memory, but no external calls.
    Argon2 {
        /// The amount of memory requested (KB).
        mem_cost: u32,
        /// The number of passes.
        time_cost: u32,
    },
    /// Infinite loop to burn cpu cycles (only run when metering is enabled)
    CpuLoop {},
    /// Infinite loop making storage calls (to test when their limit hits)
    StorageLoop {},
    /// Infinite loop reading and writing memory
    MemoryLoop {},
    /// Infinite loop sending message to itself
    MessageLoop {},
    /// Allocate large amounts of memory without consuming much gas
    AllocateLargeMemory { pages: u32 },
    /// Trigger a panic to ensure framework handles gracefully
    Panic {},
    /// In contrast to Panic, this does not use the panic handler.
    ///
    /// From <https://doc.rust-lang.org/beta/core/arch/wasm32/fn.unreachable.html>:
    /// "Generates the unreachable instruction, which causes an unconditional trap."
    Unreachable {},
    /// Returns the env for testing
    MirrorEnv {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the env for testing
    #[returns(cosmwasm_std::Env)]
    MirrorEnv {},
}
