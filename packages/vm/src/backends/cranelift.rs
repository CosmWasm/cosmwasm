#![cfg(any(feature = "cranelift", feature = "default-cranelift"))]
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime_core::{
    backend::Compiler, backend::CompilerConfig, compile_with_config, module::Module, vm::Ctx,
};

use crate::errors::VmResult;

const FAKE_GAS_AVAILABLE: u64 = 1_000_000;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let config = CompilerConfig {
        enable_verification: false, // As discussed in https://github.com/CosmWasm/cosmwasm/issues/155
        ..Default::default()
    };
    let module = compile_with_config(code, compiler().as_ref(), config)?;
    Ok(module)
}

pub fn compiler() -> Box<dyn Compiler> {
    Box::new(CraneliftCompiler::new())
}

pub fn backend() -> &'static str {
    "cranelift"
}

/// Set the amount of gas units that can be used in the context.
pub fn set_gas_limit(_ctx: &mut Ctx, _limit: u64) {}

/// Get how many more gas units can be used in the context.
pub fn get_gas_left(_ctx: &Ctx) -> u64 {
    FAKE_GAS_AVAILABLE
}
