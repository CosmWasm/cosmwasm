#![cfg(any(feature = "cranelift", feature = "default-cranelift"))]

use wasmer::Module;
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_jit::JIT;

use crate::errors::VmResult;

const FAKE_GAS_AVAILABLE: u64 = 1_000_000;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let config = CompilerConfig {
        enable_verification: false, // As discussed in https://github.com/CosmWasm/cosmwasm/issues/155
        ..Default::default()
    };
    let compiler = Cranelift::default();
    let engine = JIT::new(&mut compiler).engine();
    let store = Store::new(&engine);
    let module = Module::new(&store, code)?;
    Ok(module)
}

pub fn backend() -> &'static str {
    "cranelift"
}

/// Set the amount of gas units that can be used in the context.
pub fn set_gas_left<S: Storage, Q: Querier>(_env: &mut Env<S, Q>, _amount: u64) {}

/// Get how many more gas units can be used in the context.
pub fn get_gas_left<S: Storage, Q: Querier>(_env: &Env<S, Q>) -> u64 {
    FAKE_GAS_AVAILABLE
}
