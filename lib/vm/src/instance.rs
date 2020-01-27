use std::marker::PhantomData;

use snafu::ResultExt;
pub use wasmer_runtime_core::typed_func::Func;
use wasmer_runtime_core::{
    imports,
    module::Module,
    typed_func::{Wasm, WasmTypeList},
    vm::Ctx,
};

use cosmwasm::traits::{Api, Extern, Storage};

use crate::backends::{compile, get_gas, set_gas};
use crate::context::{
    do_canonical_address, do_human_address, do_read, do_write, leave_storage, setup_context,
    take_storage, with_storage_from_context,
};
use crate::errors::{ResolveErr, Result, RuntimeErr, WasmerErr};
use crate::memory::{read_region, write_region};

pub struct Instance<S: Storage + 'static, A: Api + 'static> {
    instance: wasmer_runtime_core::instance::Instance,
    pub api: A,
    storage: PhantomData<S>,
}

impl<S, A> Instance<S, A>
where
    S: Storage + 'static,
    A: Api + 'static,
{
    pub fn from_code(code: &[u8], deps: Extern<S, A>) -> Result<Self> {
        let module = compile(code)?;
        Instance::from_module(&module, deps)
    }

    pub fn from_module(module: &Module, deps: Extern<S, A>) -> Result<Self> {
        // copy this so it can be moved into the closures, without pulling in deps
        let api = deps.api;
        let import_obj = imports! {
            || { setup_context::<S>() },
            "env" => {
                // Reads the database entry at the given key into the the value.
                // A prepared and sufficiently large memory Region is expected at value_ptr that points to pre-allocated memory.
                // Returns length of the value in bytes on success. Returns negative value on error. An incomplete list of error codes is:
                //   value region too small: -1000002
                // Ownership of both input and output pointer is not transferred to the host.
                "read_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> i32 {
                    do_read::<S>(ctx, key_ptr, value_ptr)
                }),
                // Writes the given value into the database entry at the given key.
                // Ownership of both input and output pointer is not transferred to the host.
                "write_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| {
                    do_write::<S>(ctx, key_ptr, value_ptr)
                }),
                // Reads human address from human_ptr and writes canonicalized representation to canonical_ptr.
                // A prepared and sufficiently large memory Region is expected at canonical_ptr that points to pre-allocated memory.
                // Returns negative value on error. Returns length of the canoncal address on success.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Func::new(move |ctx: &mut Ctx, human_ptr: u32, canonical_ptr: u32| -> i32 {
                    do_canonical_address(api, ctx, human_ptr, canonical_ptr)
                }),
                // Reads canonical address from canonical_ptr and writes humanized representation to human_ptr.
                // A prepared and sufficiently large memory Region is expected at human_ptr that points to pre-allocated memory.
                // Returns negative value on error. Returns length of the human address on success.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Func::new(move |ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32| -> i32 {
                    do_human_address(api, ctx, canonical_ptr, human_ptr)
                }),
            },
        };
        let instance = module.instantiate(&import_obj).context(WasmerErr {})?;
        let res = Instance {
            instance,
            api,
            storage: PhantomData::<S> {},
        };
        res.leave_storage(Some(deps.storage));
        Ok(res)
    }

    pub fn get_gas(&self) -> u64 {
        get_gas(&self.instance)
    }

    pub fn set_gas(&mut self, gas: u64) {
        set_gas(&mut self.instance, gas)
    }

    pub fn with_storage<F: FnMut(&mut S)>(&self, func: F) {
        with_storage_from_context(self.instance.context(), func)
    }

    pub fn take_storage(&self) -> Option<S> {
        take_storage(self.instance.context())
    }

    pub fn leave_storage(&self, storage: Option<S>) {
        leave_storage(self.instance.context(), storage);
    }

    pub fn memory(&self, ptr: u32) -> Vec<u8> {
        read_region(self.instance.context(), ptr)
    }

    // allocate memory in the instance and copies the given data in
    // returns the memory offset, to be later passed as an argument
    pub fn allocate(&mut self, data: &[u8]) -> Result<u32> {
        let alloc: Func<u32, u32> = self.func("allocate")?;
        let ptr = alloc.call(data.len() as u32).context(RuntimeErr {})?;
        write_region(self.instance.context(), ptr, data)?;
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub fn deallocate(&mut self, ptr: u32) -> Result<()> {
        let dealloc: Func<u32, ()> = self.func("deallocate")?;
        dealloc.call(ptr).context(RuntimeErr {})?;
        Ok(())
    }

    pub fn func<Args, Rets>(&self, name: &str) -> Result<Func<Args, Rets, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.instance.func(name).context(ResolveErr {})
    }
}

#[cfg(test)]
mod test {
    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::mock_instance;
    use cosmwasm::mock::mock_env;
    use cosmwasm::types::coin;

    static CONTRACT_0_7: &[u8] = include_bytes!("../testdata/contract_0.7.wasm");

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn get_and_set_gas_cranelift_noop() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = instance.get_gas();
        assert!(orig_gas > 1000);
        // this is a no-op
        instance.set_gas(123456);
        assert_eq!(orig_gas, instance.get_gas());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn get_and_set_gas_singlepass_works() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = instance.get_gas();
        assert!(orig_gas > 1000000);
        // it is updated to whatever we set it with
        instance.set_gas(123456);
        assert_eq!(123456, instance.get_gas());
    }

    #[test]
    #[should_panic]
    fn with_context_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let instance = mock_instance(&CONTRACT_0_7);
        instance.with_storage(|_store| assert_eq!(1, 2));
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = 200_000;
        instance.set_gas(orig_gas);

        // init contract
        let params = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert_eq!(init_used, 70533);

        // run contract - just sanity check - results validate in contract unit tests
        instance.set_gas(orig_gas);
        let params = mock_env(
            &instance.api,
            "verifies",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = b"{}";
        let res = call_handle(&mut instance, &params, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());

        let handle_used = orig_gas - instance.get_gas();
        println!("handle used: {}", handle_used);
        assert_eq!(handle_used, 115423);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = 20_000;
        instance.set_gas(orig_gas);

        // init contract
        let params = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &params, msg);
        assert!(res.is_err());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT_0_7);
        let orig_gas = 200_000;
        instance.set_gas(orig_gas);

        // init contract
        let params = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init(&mut instance, &params, msg).unwrap().unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        instance.set_gas(orig_gas);
        // we need to encode the key in base64
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let res = call_query(&mut instance, msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer, "verifies".as_bytes());

        let query_used = orig_gas - instance.get_gas();
        println!("query used: {}", query_used);
        assert_eq!(query_used, 60315);
    }
}
