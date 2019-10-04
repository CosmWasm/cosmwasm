pub use wasmer_runtime::{Func, Instance};

use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime::{compile_with, func, imports};

use cosmwasm::mock::MockStorage;
use crate::exports::{do_read, do_write, setup_context, with_storage_from_context};

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
    module.instantiate(&import_obj).unwrap()
}

pub fn with_storage<F: FnMut(&mut MockStorage)>(instance: &Instance, func: F) {
    with_storage_from_context(instance.context(), func)
}