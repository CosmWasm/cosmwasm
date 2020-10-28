use wasmer::{Pages, Store, Target, Tunables as ReferenceTunables};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine::{Engine, Tunables};
use wasmer_engine_jit::JIT;

use super::limiting_tunables::LimitingTunables;

/// Created a store with the default compiler and the given memory limit (in pages)
pub fn make_store(memory_limit: u32) -> Store {
    let compiler = Singlepass::default();
    let engine = JIT::new(&compiler).engine();
    make_store_with_engine(&engine, memory_limit)
}

/// Created a store with no compiler and the given memory limit (in pages)
pub fn make_store_headless(memory_limit: u32) -> Store {
    let engine = JIT::headless().engine();
    make_store_with_engine(&engine, memory_limit)
}

fn make_store_with_engine(engine: &dyn Engine, memory_limit: u32) -> Store {
    let tunables = make_tunables(Pages(memory_limit));
    Store::new_with_tunables(engine, tunables)
}

fn make_tunables(memory_limit: Pages) -> impl Tunables {
    let base = ReferenceTunables::for_target(&Target::default());
    LimitingTunables::new(base, memory_limit)
}
