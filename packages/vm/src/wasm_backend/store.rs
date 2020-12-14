use std::convert::TryInto;
#[cfg(feature = "cranelift")]
use wasmer::Cranelift;
#[cfg(not(feature = "cranelift"))]
use wasmer::Singlepass;
use wasmer::{
    Engine, Pages, Store, Target,
    Tunables as ReferenceTunables, /* See https://github.com/wasmerio/wasmer/issues/1872 */
    JIT, WASM_PAGE_SIZE,
};

use crate::size::Size;

use super::limiting_tunables::LimitingTunables;

/// WebAssembly linear memory objects have sizes measured in pages. Each page
/// is 65536 (2^16) bytes. In WebAssembly version 1, a linear memory can have at
/// most 65536 pages, for a total of 2^32 bytes (4 gibibytes).
/// https://github.com/WebAssembly/memory64/blob/master/proposals/memory64/Overview.md
const MAX_WASM_MEMORY: usize = 4 * 1024 * 1024 * 1024;

/// Created a store with the default compiler and the given memory limit (in bytes)
/// If memory_limit is None, no limit is applied.
pub fn make_store(memory_limit: Option<Size>) -> Store {
    #[cfg(feature = "cranelift")]
    {
        let compiler = Cranelift::default();
        let engine = JIT::new(compiler).engine();
        make_store_with_engine(&engine, memory_limit)
    }

    #[cfg(not(feature = "cranelift"))]
    {
        let compiler = Singlepass::default();
        let engine = JIT::new(compiler).engine();
        make_store_with_engine(&engine, memory_limit)
    }
}

/// Created a store with no compiler and the given memory limit (in bytes)
/// If memory_limit is None, no limit is applied.
pub fn make_store_headless(memory_limit: Option<Size>) -> Store {
    let engine = JIT::headless().engine();
    make_store_with_engine(&engine, memory_limit)
}

/// Creates a store from an engine and an optional memory limit.
/// If no limit is set, the no custom tunables will be used.
fn make_store_with_engine(engine: &dyn Engine, memory_limit: Option<Size>) -> Store {
    match memory_limit {
        Some(limit) => {
            let capped = std::cmp::max(limit.0, MAX_WASM_MEMORY);
            // round down to ensure the limit is less than or equal to the config
            let pages: u32 = (capped / WASM_PAGE_SIZE)
                .try_into()
                .expect("Value must be <= 4 GiB/64KiB, i.e. fit in uint32. This is a bug.");
            let base = ReferenceTunables::for_target(&Target::default());
            let tunables = LimitingTunables::new(base, Pages(pages));
            Store::new_with_tunables(engine, tunables)
        }
        None => Store::new(engine),
    }
}
