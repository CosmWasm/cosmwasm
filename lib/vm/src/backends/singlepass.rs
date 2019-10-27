#![cfg(feature = "singlepass")]
use wasmer_middleware_common::metering;
use wasmer_runtime::{compile_with, Backend, Module};
use wasmer_runtime_core::codegen::{MiddlewareChain, StreamingCompiler};
use wasmer_singlepass_backend::ModuleCodeGenerator as SinglePassMCG;

static GAS_LIMIT: u64 = 10_000_000_000;

pub fn compile(code: &[u8]) -> Module {
    let c: StreamingCompiler<SinglePassMCG, _, _, _, _> = StreamingCompiler::new(move || {
        let mut chain = MiddlewareChain::new();
        chain.push(metering::Metering::new(GAS_LIMIT));
        chain
    });
    compile_with(code, &c).unwrap()
}

pub fn backend() -> Backend {
    Backend::Singlepass
}
