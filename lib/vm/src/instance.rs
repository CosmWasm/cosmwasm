pub use wasmer_runtime::Func;

use std::marker::PhantomData;
use wasmer_runtime::{func, imports, Module};
use wasmer_runtime_core::{
    error::ResolveResult,
    typed_func::{Wasm, WasmTypeList},
};

use crate::backends::compile;
use crate::context::{
    do_read, do_write, leave_storage, setup_context, take_storage, with_storage_from_context,
};
use crate::memory::{read_memory, write_memory};
use cosmwasm::storage::Storage;

pub struct Instance<T: Storage + 'static> {
    instance: wasmer_runtime::Instance,
    storage: PhantomData<T>,
}

impl<T> Instance<T>
where
    T: Storage + 'static,
{
    pub fn from_code(code: &[u8], storage: T) -> Instance<T> {
        let module = compile(code);
        Instance::from_module(&module, storage)
    }

    pub fn from_module(module: &Module, storage: T) -> Instance<T> {
        let import_obj = imports! {
            || { setup_context::<T>() },
            "env" => {
                "c_read" => func!(do_read::<T>),
                "c_write" => func!(do_write::<T>),
            },
        };

        // TODO: add metering options here
        // TODO: we unwrap rather than Result as:
        //   the trait `std::marker::Send` is not implemented for `(dyn std::any::Any + 'static)`
        // convert from wasmer error to failure error....
        let instance = module.instantiate(&import_obj).unwrap();
        let res = Instance {
            instance,
            storage: PhantomData::<T>::default(),
        };
        res.leave_storage(Some(storage));
        res
    }

    pub fn with_storage<F: FnMut(&mut T)>(&self, func: F) {
        with_storage_from_context(self.instance.context(), func)
    }

    pub fn take_storage(&self) -> Option<T> {
        take_storage(self.instance.context())
    }

    pub fn leave_storage(&self, storage: Option<T>) {
        leave_storage(self.instance.context(), storage);
    }

    pub fn memory(&self, ptr: u32) -> Vec<u8> {
        read_memory(self.instance.context(), ptr)
    }

    // write_mem allocates memory in the instance and copies the given data in
    // returns the memory offset, to be passed as an argument
    // panics on any error (TODO, use result?)
    pub fn allocate(&mut self, data: &[u8]) -> u32 {
        let alloc: Func<(u32), (u32)> = self.func("allocate").unwrap();
        let ptr = alloc.call(data.len() as u32).unwrap();
        write_memory(self.instance.context(), ptr, data);
        ptr
    }

    pub fn func<Args, Rets>(&self, name: &str) -> ResolveResult<Func<Args, Rets, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.instance.func(name)
    }
}
