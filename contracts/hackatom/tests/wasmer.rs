pub use wasmer_runtime::{Func, func, imports};

use wasmer_runtime::{compile_with, Func, ImportObject, Instance};
use wasmer_clif_backend::CraneliftCompiler;

fn wasm_instance(code: &[u8], import_obj: &ImportObject) -> Instance {
    // TODO: add caching here!
    // TODO: add metering options here
    let module = compile_with(wasm, &CraneliftCompiler::new()).unwrap();
    module.instantiate (import_obj).unwrap()
}
