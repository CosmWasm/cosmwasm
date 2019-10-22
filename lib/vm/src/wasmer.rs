pub use wasmer_runtime::{Func, Instance};

use wasmer_runtime::{func, imports, Module};

use crate::backends::compile;
use crate::exports::{do_read, do_write, setup_context, with_storage_from_context};
use cosmwasm::storage::Storage;

pub fn instantiate<T>(code: &[u8], storage: T) -> Instance
    where T: Storage + Send + Sync + 'static {
    let module = compile(code);
    mod_to_instance(&module, storage)
}

pub fn mod_to_instance<T>(module: &Module, storage: T) -> Instance
  where T: Storage + Send + Sync + 'static {
    let import_obj = imports! {
        move || { setup_context(storage) },
        "env" => {
            "c_read" => func!(do_read),
            "c_write" => func!(do_write),
        },
    };

    // TODO: add metering options here
    // TODO: we unwrap rather than Result as:
    //   the trait `std::marker::Send` is not implemented for `(dyn std::any::Any + 'static)`
    // convert from wasmer error to failure error....
    module.instantiate(&import_obj).unwrap()
}

pub fn with_storage<T: Storage, F: FnMut(&mut T)>(instance: &Instance, func: F) {
    with_storage_from_context::<T>(instance.context(), func)
}