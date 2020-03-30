use std::marker::PhantomData;

use snafu::ResultExt;
pub use wasmer_runtime_core::typed_func::Func;
use wasmer_runtime_core::{
    imports,
    module::Module,
    typed_func::{Wasm, WasmTypeList},
    vm::Ctx,
};

use cosmwasm_std::{Api, Extern, Storage};

use crate::backends::{compile, get_gas, set_gas};
use crate::context::{
    do_canonical_address, do_human_address, do_read, do_remove, do_write, leave_storage,
    setup_context, take_storage, with_storage_from_context,
};
#[cfg(feature = "iterator")]
use crate::context::{do_next, do_scan};
use crate::errors::{ResolveErr, Result, RuntimeErr, WasmerErr};
use crate::memory::{read_region, write_region};

pub struct Instance<S: Storage + 'static, A: Api + 'static> {
    wasmer_instance: wasmer_runtime_core::instance::Instance,
    pub api: A,
    // This does not store data but only fixes type information
    type_storage: PhantomData<S>,
}

impl<S, A> Instance<S, A>
where
    S: Storage + 'static,
    A: Api + 'static,
{
    pub fn from_code(code: &[u8], deps: Extern<S, A>, gas_limit: u64) -> Result<Self> {
        let module = compile(code)?;
        Instance::from_module(&module, deps, gas_limit)
    }

    pub fn from_module(module: &Module, deps: Extern<S, A>, gas_limit: u64) -> Result<Self> {
        // copy this so it can be moved into the closures, without pulling in deps
        let api = deps.api;
        let import_obj = imports! {
            || { setup_context::<S>() },
            "env" => {
                // Reads the database entry at the given key into the the value.
                // A prepared and sufficiently large memory Region is expected at value_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error. An incomplete list of error codes is:
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
                // Removes the value at the given key. Different than writing &[] as future
                // scans will not find this key.
                // Ownership of both key pointer is not transferred to the host.
                "remove_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32| {
                    do_remove::<S>(ctx, key_ptr)
                }),
                // Creates an iterator that will go from start to end
                // Order is defined in cosmwasm::traits::Order and may be 1/Ascending or 2/Descending.
                // Ownership of both start and end pointer is not transferred to the host.
                // Returns negative code on error, 0 on success
                "scan_db" => Func::new(move |ctx: &mut Ctx, start_ptr: u32, end_ptr: u32, order: i32| -> i32{
                    #[cfg(not(feature = "iterator"))]
                    {
                        // get rid of unused argument warning
                        let (_, _, _, _) = (ctx, start_ptr, end_ptr, order);
                        return 0;
                    }
                    #[cfg(feature = "iterator")]
                    do_scan::<S>(ctx, start_ptr, end_ptr, order)
                }),
                // Creates an iterator that will go from start to end
                // Order is defined in cosmwasm::traits::Order and may be 1/Ascending or 2/Descending.
                // Ownership of both start and end pointer is not transferred to the host.
                "next_db" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> i32 {
                    #[cfg(not(feature = "iterator"))]
                    {
                        // get rid of unused argument warning
                        let (_, _, _) = (ctx, key_ptr, value_ptr);
                        return 0;
                    }
                    #[cfg(feature = "iterator")]
                    do_next::<S>(ctx, key_ptr, value_ptr)
                }),
                // Reads human address from human_ptr and writes canonicalized representation to canonical_ptr.
                // A prepared and sufficiently large memory Region is expected at canonical_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Func::new(move |ctx: &mut Ctx, human_ptr: u32, canonical_ptr: u32| -> i32 {
                    do_canonical_address(api, ctx, human_ptr, canonical_ptr)
                }),
                // Reads canonical address from canonical_ptr and writes humanized representation to human_ptr.
                // A prepared and sufficiently large memory Region is expected at human_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Func::new(move |ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32| -> i32 {
                    do_human_address(api, ctx, canonical_ptr, human_ptr)
                }),
            },
        };
        let wasmer_instance = module.instantiate(&import_obj).context(WasmerErr {})?;
        Ok(Instance::from_wasmer(wasmer_instance, deps, gas_limit))
    }

    pub fn from_wasmer(
        mut wasmer_instance: wasmer_runtime_core::Instance,
        deps: Extern<S, A>,
        gas_limit: u64,
    ) -> Self {
        set_gas(&mut wasmer_instance, gas_limit);
        leave_storage(wasmer_instance.context(), Some(deps.storage));
        Instance {
            wasmer_instance: wasmer_instance,
            api: deps.api,
            type_storage: PhantomData::<S> {},
        }
    }

    /// Takes ownership of instance and decomposes it into its components.
    /// The components we want to preserve are returned, the rest is dropped.
    pub fn recycle(instance: Self) -> (wasmer_runtime_core::Instance, Option<Extern<S, A>>) {
        let ext = if let Some(storage) = take_storage(instance.wasmer_instance.context()) {
            Some(Extern {
                storage: storage,
                api: instance.api,
            })
        } else {
            None
        };
        (instance.wasmer_instance, ext)
    }

    /// Returns the currently remaining gas
    pub fn get_gas(&self) -> u64 {
        get_gas(&self.wasmer_instance)
    }

    pub fn with_storage<F: FnMut(&mut S)>(&self, func: F) {
        with_storage_from_context(self.wasmer_instance.context(), func)
    }

    /// Copies all data described by the Region at the given pointer from Wasm to the caller.
    pub(crate) fn memory(&self, region_ptr: u32) -> Vec<u8> {
        read_region(self.wasmer_instance.context(), region_ptr)
    }

    /// Allocates memory in the instance and copies the given data into it.
    /// Returns a pointer in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, data: &[u8]) -> Result<u32> {
        let alloc: Func<u32, u32> = self.func("allocate")?;
        let ptr = alloc.call(data.len() as u32).context(RuntimeErr {})?;
        write_region(self.wasmer_instance.context(), ptr, data)?;
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> Result<()> {
        let dealloc: Func<u32, ()> = self.func("deallocate")?;
        dealloc.call(ptr).context(RuntimeErr {})?;
        Ok(())
    }

    pub(crate) fn func<Args, Rets>(&self, name: &str) -> Result<Func<Args, Rets, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.wasmer_instance.func(name).context(ResolveErr {})
    }
}

#[cfg(test)]
mod test {
    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::{mock_instance, mock_instance_with_gas_limit};
    use cosmwasm_std::coin;
    use cosmwasm_std::testing::mock_env;

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn set_get_and_gas_cranelift_noop() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 1_000_000);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn set_get_and_gas_singlepass_works() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    #[should_panic]
    fn with_context_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let instance = mock_instance(&CONTRACT);
        instance.with_storage(|_store| assert_eq!(1, 2));
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT);
        let orig_gas = instance.get_gas();

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init(&mut instance, &env, msg).unwrap();

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert_eq!(init_used, 46470);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_deducts_gas_handle() {
        let mut instance = mock_instance(&CONTRACT);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init(&mut instance, &env, msg).unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_handle = instance.get_gas();
        let env = mock_env(
            &instance.api,
            "verifies",
            &coin("15", "earth"),
            &coin("1015", "earth"),
        );
        let msg = br#"{"release":{}}"#;
        call_handle(&mut instance, &env, msg).unwrap();

        let handle_used = gas_before_handle - instance.get_gas();
        println!("handle used: {}", handle_used);
        assert_eq!(handle_used, 59376);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, 20_000);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init(&mut instance, &env, msg);
        assert!(res.is_err());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT);

        // init contract
        let env = mock_env(&instance.api, "creator", &coin("1000", "earth"), &[]);
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init(&mut instance, &env, msg).unwrap().unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_query = instance.get_gas();
        // we need to encode the key in base64
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let res = call_query(&mut instance, msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer.as_slice(), "verifies".as_bytes());

        let query_used = gas_before_query - instance.get_gas();
        println!("query used: {}", query_used);
        assert_eq!(query_used, 19819);
    }
}
