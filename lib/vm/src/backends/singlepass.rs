#![cfg(any(feature = "singlepass", feature = "default-singlepass"))]
use wasmer_middleware_common::metering;
use wasmer_runtime::{compile_with, Backend, Compiler, Instance, Module};
use wasmer_runtime_core::codegen::{MiddlewareChain, StreamingCompiler};
use wasmer_singlepass_backend::ModuleCodeGenerator as SinglePassMCG;

use crate::errors::{CompileErr, Error};
use crate::middleware::DeterministicMiddleware;
use snafu::ResultExt;

static GAS_LIMIT: u64 = 10_000_000_000;

pub fn compile(code: &[u8]) -> Result<Module, Error> {
    compile_with(code, compiler().as_ref()).context(CompileErr {})
}

pub fn compiler() -> Box<dyn Compiler> {
    let c: StreamingCompiler<SinglePassMCG, _, _, _, _> = StreamingCompiler::new(move || {
        let mut chain = MiddlewareChain::new();
        chain.push(DeterministicMiddleware::new());
        chain.push(metering::Metering::new(GAS_LIMIT));
        chain
    });
    Box::new(c)
}

pub fn backend() -> Backend {
    Backend::Singlepass
}

pub fn set_gas(instance: &mut Instance, limit: u64) {
    let used = GAS_LIMIT - limit;
    metering::set_points_used(instance, used)
}

pub fn get_gas(instance: &Instance) -> u64 {
    let used = metering::get_points_used(instance);
    GAS_LIMIT - used
}
