#![cfg(feature = "cranelift")]
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime::{compile_with, Backend, Instance, Module};

static FAKE_GAS_AVAILBLE: u64 = 1_000_000;

pub fn compile(code: &[u8]) -> Module {
    compile_with(code, &CraneliftCompiler::new()).unwrap()
}

pub fn backend() -> Backend {
    Backend::Cranelift
}

pub fn set_gas(_instance: &mut Instance, _limit: u64) {}

pub fn get_gas(_instance: &Instance) -> u64 {
    FAKE_GAS_AVAILABLE
}
