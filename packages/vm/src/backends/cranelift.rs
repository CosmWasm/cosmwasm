#![cfg(any(feature = "cranelift", feature = "default-cranelift"))]
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime_core::{backend::Compiler, compile_with, instance::Instance, module::Module};

use crate::errors::{CompileErr, Error};
use snafu::ResultExt;

static FAKE_GAS_AVAILABLE: u64 = 1_000_000;

pub fn compile(code: &[u8]) -> Result<Module, Error> {
    compile_with(code, compiler().as_ref()).context(CompileErr {})
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
