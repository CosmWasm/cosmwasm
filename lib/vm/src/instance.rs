pub use wasmer_runtime::Func;

use snafu::ResultExt;
use std::marker::PhantomData;
use wasmer_runtime::{func, imports, Module};
use wasmer_runtime_core::typed_func::{Wasm, WasmTypeList};

use crate::backends::{compile, get_gas, set_gas};
use crate::context::{
    do_read, do_write, leave_storage, setup_context, take_storage, with_storage_from_context,
};
use crate::errors::{Error, ResolveErr, RuntimeErr, WasmerErr};
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
    pub fn from_code(code: &[u8], storage: T) -> Result<Instance<T>, Error> {
        let module = compile(code)?;
        Instance::from_module(&module, storage)
    }

    pub fn from_module(module: &Module, storage: T) -> Result<Instance<T>, Error> {
        let import_obj = imports! {
            || { setup_context::<T>() },
            "env" => {
                "c_read" => func!(do_read::<T>),
                "c_write" => func!(do_write::<T>),
            },
        };
        let instance = module.instantiate(&import_obj).context(WasmerErr {})?;
        let res = Instance {
            instance,
            storage: PhantomData::<T>::default(),
        };
        res.leave_storage(Some(storage));
        Ok(res)
    }

    pub fn get_gas(&self) -> u64 {
        get_gas(&self.instance)
    }

    pub fn set_gas(&mut self, gas: u64) {
        set_gas(&mut self.instance, gas)
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
    pub fn allocate(&mut self, data: &[u8]) -> Result<u32, Error> {
        let alloc: Func<(u32), (u32)> = self.func("allocate")?;
        let ptr = alloc.call(data.len() as u32).context(RuntimeErr {})?;
        write_memory(self.instance.context(), ptr, data);
        Ok(ptr)
    }

    pub fn func<Args, Rets>(&self, name: &str) -> Result<Func<Args, Rets, Wasm>, Error>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.instance.func(name).context(ResolveErr {})
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::calls::{call_handle, call_init};
    use cosmwasm::mock::MockStorage;
    use cosmwasm::types::{coin, mock_params};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn get_and_set_gas_cranelift_noop() {
        let storage = MockStorage::new();
        let mut instance = Instance::from_code(CONTRACT, storage).unwrap();
        let orig_gas = instance.get_gas();
        assert!(orig_gas > 1000);
        // this is a no-op
        instance.set_gas(123456);
        assert_eq!(orig_gas, instance.get_gas());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn get_and_set_gas_singlepass_works() {
        let storage = MockStorage::new();
        let mut instance = Instance::from_code(CONTRACT, storage).unwrap();
        let orig_gas = instance.get_gas();
        assert!(orig_gas > 1000000);
        // it is updated to whatever we set it with
        instance.set_gas(123456);
        assert_eq!(123456, instance.get_gas());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas() {
        let storage = MockStorage::new();
        let mut instance = Instance::from_code(CONTRACT, storage).unwrap();
        let orig_gas = 200_000;
        instance.set_gas(orig_gas);

        // init contract
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert!(init_used > 30_000);

        // run contract - just sanity check - results validate in contract unit tests
        let params = mock_params("verifies", &coin("15", "earth"), &coin("1015", "earth"));
        let msg = b"{}";
        let res = call_handle(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());

        let total_used = orig_gas - instance.get_gas();
        println!("total used: {}", total_used);
        assert!(total_used > 100_000);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_enforces_gas_limit() {
        let storage = MockStorage::new();
        let mut instance = Instance::from_code(CONTRACT, storage).unwrap();
        let orig_gas = 20_000;
        instance.set_gas(orig_gas);

        // init contract
        let params = mock_params("creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        // this call will panic on out-of-gas
        // TODO: improve error handling through-out the whole stack
        let res = call_init(&mut instance, &params, msg);
        assert!(res.is_err());
    }
}
