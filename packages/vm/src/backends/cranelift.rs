#![cfg(any(feature = "cranelift", feature = "default-cranelift"))]
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime_core::{
    backend::Compiler, backend::CompilerConfig, compile_with_config, instance::Instance,
    module::Module,
};

use crate::errors::{CompileErr, VmResult};
use snafu::ResultExt;

static FAKE_GAS_AVAILABLE: u64 = 1_000_000;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let config = CompilerConfig {
        enable_verification: false, // As discussed in https://github.com/CosmWasm/cosmwasm/issues/155
        ..Default::default()
    };
    compile_with_config(code, compiler().as_ref(), config).context(CompileErr {})
}

pub fn compiler() -> Box<dyn Compiler> {
    Box::new(CraneliftCompiler::new())
}

pub fn backend() -> &'static str {
    "cranelift"
}

pub fn set_gas(_instance: &mut Instance, _limit: u64) {}

pub fn get_gas(_instance: &Instance) -> u64 {
    FAKE_GAS_AVAILABLE
}
