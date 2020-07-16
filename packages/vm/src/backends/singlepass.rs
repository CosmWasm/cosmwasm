#![cfg(any(feature = "singlepass", feature = "default-singlepass"))]
use wasmer_middleware_common::metering;
use wasmer_runtime_core::{
    backend::Compiler,
    codegen::{MiddlewareChain, StreamingCompiler},
    compile_with,
    module::Module,
    vm::Ctx,
};
use wasmer_singlepass_backend::ModuleCodeGenerator as SinglePassMCG;

use crate::errors::VmResult;
use crate::middleware::DeterministicMiddleware;

/// In Wasmer, the gas limit is set on modules during compilation and is included in the cached modules.
/// This causes issues when trying to instantiate the same compiled module with a different gas limit.
/// A fix for this is proposed here: https://github.com/wasmerio/wasmer/pull/996.
///
/// To work around this limitation, we set the gas limit of all Wasmer instances to this very high value,
/// assuming users won't request more than this amount of gas. In order to set the real gas limit, we pretend
/// to consume the difference between the two in `set_gas_limit` ("points used" in the metering middleware).
/// Since we observed overflow behaviour in the points used, we ensure both MAX_GAS_LIMIT and points used stay
/// far below u64::MAX.
const MAX_GAS_LIMIT: u64 = u64::MAX / 2;

pub fn compile(code: &[u8]) -> VmResult<Module> {
    let module = compile_with(code, compiler().as_ref())?;
    Ok(module)
}

pub fn compiler() -> Box<dyn Compiler> {
    let c: StreamingCompiler<SinglePassMCG, _, _, _, _> = StreamingCompiler::new(move || {
        let mut chain = MiddlewareChain::new();
        chain.push(DeterministicMiddleware::new());
        chain.push(metering::Metering::new(MAX_GAS_LIMIT));
        chain
    });
    Box::new(c)
}

pub fn backend() -> &'static str {
    "singlepass"
}

/// Set the amount of gas units that can be used in the context.
pub fn set_gas_limit(ctx: &mut Ctx, limit: u64) {
    if limit > MAX_GAS_LIMIT {
        panic!(
            "Attempted to set gas limit larger than max gas limit (got: {}; maximum: {}).",
            limit, MAX_GAS_LIMIT
        );
    } else {
        let used = MAX_GAS_LIMIT - limit;
        metering::set_points_used_ctx(ctx, used);
    }
}

/// Get how many more gas units can be used in the context.
pub fn get_gas_left(ctx: &Ctx) -> u64 {
    let used = metering::get_points_used_ctx(ctx);
    // when running out of gas, get_points_used can exceed MAX_GAS_LIMIT
    MAX_GAS_LIMIT.saturating_sub(used)
}

#[cfg(test)]
mod test {
    use super::*;
    use wabt::wat2wasm;
    use wasmer_runtime_core::{imports, Instance as WasmerInstance};

    fn instantiate(code: &[u8]) -> WasmerInstance {
        let module = compile(code).unwrap();
        let import_obj = imports! { "env" => {}, };
        module.instantiate(&import_obj).unwrap()
    }

    #[test]
    fn get_gas_left_defaults_to_constant() {
        let wasm = wat2wasm("(module)").unwrap();
        let instance = instantiate(&wasm);
        let gas_left = get_gas_left(instance.context());
        assert_eq!(gas_left, MAX_GAS_LIMIT);
    }

    #[test]
    fn set_gas_limit_works() {
        let wasm = wat2wasm("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let limit = 3456789;
        set_gas_limit(instance.context_mut(), limit);
        assert_eq!(get_gas_left(instance.context()), limit);

        let limit = 1;
        set_gas_limit(instance.context_mut(), limit);
        assert_eq!(get_gas_left(instance.context()), limit);

        let limit = 0;
        set_gas_limit(instance.context_mut(), limit);
        assert_eq!(get_gas_left(instance.context()), limit);

        let limit = MAX_GAS_LIMIT;
        set_gas_limit(instance.context_mut(), limit);
        assert_eq!(get_gas_left(instance.context()), limit);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to set gas limit larger than max gas limit (got: 9223372036854775808; maximum: 9223372036854775807)."
    )]
    fn set_gas_limit_panic_for_values_too_large() {
        let wasm = wat2wasm("(module)").unwrap();
        let mut instance = instantiate(&wasm);

        let limit = MAX_GAS_LIMIT + 1;
        set_gas_limit(instance.context_mut(), limit);
    }
}
