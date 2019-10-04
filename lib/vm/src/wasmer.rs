pub use wasmer_runtime::{Func};

use wasmer_runtime::{compile_with, func, imports, Instance};
use wasmer_clif_backend::CraneliftCompiler;

use crate::exports::{do_read, do_write, setup_context};

pub fn instantiate(code: &[u8]) -> Instance {
    let import_obj = imports! {
        || { setup_context() },
        "env" => {
            "c_read" => func!(do_read),
            "c_write" => func!(do_write),
        },
    };

    // TODO: add caching here!
    // TODO: add metering options here
    let module = compile_with(code, &CraneliftCompiler::new()).unwrap();
    module.instantiate (&import_obj).unwrap()
}

