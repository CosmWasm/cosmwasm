#![cfg(feature = "singlepass")]
use wasmer_singlepass_backend::SinglePassCompiler;
use wasmer_runtime::{compile_with, Backend, Module};

pub fn compile(code: &[u8]) -> Module {
    compile_with(code, &SinglePassCompiler::new()).unwrap()
}

pub fn backend() -> Backend {
    Backend::Singlepass
}
