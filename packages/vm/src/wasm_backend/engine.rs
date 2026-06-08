use super::{is_accounting, Gatekeeper, LimitingTunables, Metering};
use crate::Size;
use cosmwasm_vm_derive::hash_function;
use std::sync::Arc;
use wasmer::{
    sys::BaseTunables, wasmparser::Operator, CompilerConfig, Engine, NativeEngineExt, Pages,
    Target, WASM_PAGE_SIZE,
};

/// WebAssembly linear memory objects have sizes measured in pages.
/// Each page is 65536 (2^16) bytes. In WebAssembly version 1, a linear memory
/// can have at most 65536 pages, for a total of 2^32 bytes (4 gibibytes).
/// https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md
const MAX_WASM_PAGES: u32 = 65536;

//-----------------------------------------------------------------------------
// Cost function.
//
// This function is hashed and put into the `raw_module_version_discriminator`
// because it is used as a part of the compilation process.
// If this function changes, all modules need to be recompiled.
// Gas calculation procedure is explained in details here:
//   https://cosmwasm.github.io/core/architecture/gas
//-----------------------------------------------------------------------------
#[hash_function(const_name = "COST_FUNCTION_HASH")]
fn cost(operator: &Operator) -> u64 {
    const GAS_PER_OPERATION: u64 = 115;
    if is_accounting(operator) {
        GAS_PER_OPERATION * 14
    } else {
        GAS_PER_OPERATION
    }
}

/// Creates a compiler config using Wasmer Singlepass.
pub fn make_compiler_config() -> impl CompilerConfig + Into<Engine> {
    wasmer::Singlepass::new()
}

/// Creates an engine without a compiler.
/// This is used to run modules compiled before.
pub fn make_runtime_engine(memory_limit: Option<Size>) -> Engine {
    let mut engine = Engine::headless();
    if let Some(limit) = memory_limit {
        let base = BaseTunables::for_target(&Target::default());
        let tunables = LimitingTunables::new(base, limit_to_pages(limit));
        engine.set_tunables(tunables);
    }
    engine
}

/// Creates an Engine with a compiler attached. Use this when compiling Wasm to a module.
pub fn make_compiling_engine(memory_limit: Option<Size>) -> Engine {
    let gas_limit = 0;
    let deterministic = Arc::new(Gatekeeper::default());
    let metering = Arc::new(Metering::new(gas_limit, cost));

    let mut compiler = make_compiler_config();
    compiler.canonicalize_nans(true);
    compiler.push_middleware(deterministic);
    compiler.push_middleware(metering);
    let mut engine: Engine = compiler.into();
    if let Some(limit) = memory_limit {
        let base = BaseTunables::for_target(&Target::default());
        let tunables = LimitingTunables::new(base, limit_to_pages(limit));
        engine.set_tunables(tunables);
    }
    engine
}

fn limit_to_pages(limit: Size) -> Pages {
    // round down to ensure the limit is less than or equal to the config
    let limit_in_pages: usize = limit.0 / WASM_PAGE_SIZE;

    let capped = match u32::try_from(limit_in_pages) {
        Ok(x) => std::cmp::min(x, MAX_WASM_PAGES),
        // The only case where TryFromIntError can happen is when
        // limit_in_pages exceeds the u32 range. In this case it is way
        // larger than MAX_WASM_PAGES and needs to be capped.
        Err(_too_large) => MAX_WASM_PAGES,
    };
    Pages(capped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_works() {
        // accounting operator
        assert_eq!(cost(&Operator::Br { relative_depth: 3 }), 1610);
        assert_eq!(cost(&Operator::Return {}), 1610);

        // anything else
        assert_eq!(cost(&Operator::I64Const { value: 7 }), 115);
        assert_eq!(cost(&Operator::I64Extend8S {}), 115);
    }

    #[test]
    fn make_compiler_config_returns_singlepass() {
        let cc = Box::new(make_compiler_config());
        assert_eq!(cc.compiler().name(), "singlepass");
    }

    #[test]
    fn limit_to_pages_works() {
        // rounds down
        assert_eq!(limit_to_pages(Size::new(0)), Pages(0));
        assert_eq!(limit_to_pages(Size::new(1)), Pages(0));
        assert_eq!(limit_to_pages(Size::kibi(63)), Pages(0));
        assert_eq!(limit_to_pages(Size::kibi(64)), Pages(1));
        assert_eq!(limit_to_pages(Size::kibi(65)), Pages(1));
        assert_eq!(limit_to_pages(Size::new(u32::MAX as usize)), Pages(65535));
        // caps at 4 GiB
        assert_eq!(limit_to_pages(Size::gibi(3)), Pages(49152));
        assert_eq!(limit_to_pages(Size::gibi(4)), Pages(65536));
        assert_eq!(limit_to_pages(Size::gibi(5)), Pages(65536));
        assert_eq!(limit_to_pages(Size::new(usize::MAX)), Pages(65536));
    }
}
