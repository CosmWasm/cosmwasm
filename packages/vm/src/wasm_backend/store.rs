#[cfg(feature = "cranelift")]
use wasmer::Cranelift;
#[cfg(not(feature = "cranelift"))]
use wasmer::Singlepass;
use wasmer::{
    Bytes, Engine, Pages, Store, Target,
    Tunables as ReferenceTunables, /* See https://github.com/wasmerio/wasmer/issues/1872 */
    JIT,
};

use crate::size::Size;

use super::limiting_tunables::LimitingTunables;

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
            let pages = Pages::from(Bytes(limit.0));
            let base = ReferenceTunables::for_target(&Target::default());
            let tunables = LimitingTunables::new(base, pages);
            Store::new_with_tunables(engine, tunables)
        }
        None => Store::new(engine),
    }
}
