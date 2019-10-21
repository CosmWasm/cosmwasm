pub use wasmer_runtime::{Func, Instance};

use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime::{compile_with, func, imports};

use crate::exports::{do_read, do_write, setup_context, with_storage_from_context};
use cosmwasm::mock::MockStorage;

pub fn instantiate(code: &[u8]) -> Instance {
    let import_obj = imports! {
        || { setup_context() },
        "env" => {
            "c_read" => func!(do_read),
            "c_write" => func!(do_write),
        },
    };

    // TODO: add metering options here
    // TODO: we unwrap rather than Result as:
    //   the trait `std::marker::Send` is not implemented for `(dyn std::any::Any + 'static)`
    // convert from wasmer error to failure error....
    let module = compile_with(code, &CraneliftCompiler::new()).unwrap();
    module.instantiate(&import_obj).unwrap()
}

pub fn with_storage<F: FnMut(&mut MockStorage)>(instance: &Instance, func: F) {
    with_storage_from_context(instance.context(), func)
}
