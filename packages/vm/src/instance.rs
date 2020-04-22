use std::marker::PhantomData;

use snafu::ResultExt;
pub use wasmer_runtime_core::typed_func::Func;
use wasmer_runtime_core::{
    imports,
    module::Module,
    typed_func::{Wasm, WasmTypeList},
    vm::Ctx,
};

use cosmwasm_std::{Api, Extern, Querier, Storage};

use crate::backends::{compile, get_gas, set_gas};
use crate::context::{
    move_into_context, move_out_of_context, setup_context, with_storage_from_context,
};
use crate::conversion::to_u32;
use crate::errors::{ResolveErr, VmResult, WasmerErr, WasmerRuntimeErr};
use crate::imports::{
    do_canonicalize_address, do_humanize_address, do_query_chain, do_read, do_remove, do_write,
};
#[cfg(feature = "iterator")]
use crate::imports::{do_next, do_scan};
use crate::memory::{get_memory_info, read_region, write_region};

static WASM_PAGE_SIZE: u64 = 64 * 1024;

pub struct Instance<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static> {
    wasmer_instance: wasmer_runtime_core::instance::Instance,
    pub api: A,
    // This does not store data but only fixes type information
    type_storage: PhantomData<S>,
    type_querier: PhantomData<Q>,
}

impl<S, A, Q> Instance<S, A, Q>
where
    S: Storage + 'static,
    A: Api + 'static,
    Q: Querier + 'static,
{
    pub fn from_code(code: &[u8], deps: Extern<S, A, Q>, gas_limit: u64) -> VmResult<Self> {
        let module = compile(code)?;
        Instance::from_module(&module, deps, gas_limit)
    }

    pub fn from_module(module: &Module, deps: Extern<S, A, Q>, gas_limit: u64) -> VmResult<Self> {
        let mut import_obj = imports! { || { setup_context::<S, Q>() }, "env" => {}, };

        // copy this so it can be moved into the closures, without pulling in deps
        let api = deps.api;
        import_obj.extend(imports! {
            "env" => {
                // Reads the database entry at the given key into the the value.
                // A prepared and sufficiently large memory Region is expected at value_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error. An incomplete list of error codes is:
                //   value region too small: -1_000_001
                //   key does not exist: -1_001_001
                // Ownership of both input and output pointer is not transferred to the host.
                "db_read" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> i32 {
                    do_read::<S, Q>(ctx, key_ptr, value_ptr)
                }),
                // Writes the given value into the database entry at the given key.
                // Ownership of both input and output pointer is not transferred to the host.
                // Returns 0 on success. Returns negative value on error.
                "db_write" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> i32 {
                    do_write::<S, Q>(ctx, key_ptr, value_ptr)
                }),
                // Removes the value at the given key. Different than writing &[] as future
                // scans will not find this key.
                // At the moment it is not possible to differentiate between a key that existed before and one that did not exist (https://github.com/CosmWasm/cosmwasm/issues/290).
                // Ownership of both key pointer is not transferred to the host.
                // Returns 0 on success. Returns negative value on error.
                "db_remove" => Func::new(move |ctx: &mut Ctx, key_ptr: u32| -> i32 {
                    do_remove::<S, Q>(ctx, key_ptr)
                }),
                // Reads human address from human_ptr and writes canonicalized representation to canonical_ptr.
                // A prepared and sufficiently large memory Region is expected at canonical_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Func::new(move |ctx: &mut Ctx, human_ptr: u32, canonical_ptr: u32| -> i32 {
                    do_canonicalize_address(api, ctx, human_ptr, canonical_ptr)
                }),
                // Reads canonical address from canonical_ptr and writes humanized representation to human_ptr.
                // A prepared and sufficiently large memory Region is expected at human_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Func::new(move |ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32| -> i32 {
                    do_humanize_address(api, ctx, canonical_ptr, human_ptr)
                }),
                "query_chain" => Func::new(move |ctx: &mut Ctx, request_ptr: u32, response_ptr: u32| -> i32 {
                    do_query_chain::<S, Q>(ctx, request_ptr, response_ptr)
                }),
            },
        });

        #[cfg(feature = "iterator")]
        import_obj.extend(imports! {
            "env" => {
                // Creates an iterator that will go from start to end
                // Order is defined in cosmwasm::traits::Order and may be 1/Ascending or 2/Descending.
                // Ownership of both start and end pointer is not transferred to the host.
                // Returns negative code on error.
                // Returns iterator ID > 0 on success.
                "db_scan" => Func::new(move |ctx: &mut Ctx, start_ptr: u32, end_ptr: u32, order: i32| -> i32 {
                    do_scan::<S, Q>(ctx, start_ptr, end_ptr, order)
                }),
                // Get next element of iterator with ID `iterator_id`.
                // Expectes Regions in key_ptr and value_ptr, in which the result is written.
                // An empty key value (Region of length 0) means no more element.
                // Ownership of both key and value pointer is not transferred to the host.
                "db_next" => Func::new(move |ctx: &mut Ctx, iterator_id: u32, key_ptr: u32, value_ptr: u32| -> i32 {
                    do_next::<S, Q>(ctx, iterator_id, key_ptr, value_ptr)
                }),
            },
        });

        let wasmer_instance = module.instantiate(&import_obj).context(WasmerErr {})?;
        Ok(Instance::from_wasmer(wasmer_instance, deps, gas_limit))
    }

    pub fn from_wasmer(
        mut wasmer_instance: wasmer_runtime_core::Instance,
        deps: Extern<S, A, Q>,
        gas_limit: u64,
    ) -> Self {
        set_gas(&mut wasmer_instance, gas_limit);
        move_into_context(wasmer_instance.context_mut(), deps.storage, deps.querier);
        Instance {
            wasmer_instance,
            api: deps.api,
            type_storage: PhantomData::<S> {},
            type_querier: PhantomData::<Q> {},
        }
    }

    /// Takes ownership of instance and decomposes it into its components.
    /// The components we want to preserve are returned, the rest is dropped.
    pub fn recycle(mut instance: Self) -> (wasmer_runtime_core::Instance, Option<Extern<S, A, Q>>) {
        let ext = if let (Some(storage), Some(querier)) =
            move_out_of_context(instance.wasmer_instance.context_mut())
        {
            Some(Extern {
                storage,
                api: instance.api,
                querier,
            })
        } else {
            None
        };
        (instance.wasmer_instance, ext)
    }

    /// Returns the size of the default memory in bytes.
    /// This provides a rough idea of the peak memory consumption. Note that
    /// Wasm memory always grows in 64 KiB steps (pages) and can never shrink
    /// (https://github.com/WebAssembly/design/issues/1300#issuecomment-573867836).
    pub fn get_memory_size(&self) -> u64 {
        (get_memory_info(self.wasmer_instance.context()).size as u64) * WASM_PAGE_SIZE
    }

    /// Returns the currently remaining gas
    pub fn get_gas(&self) -> u64 {
        get_gas(&self.wasmer_instance)
    }

    pub fn with_storage<F: FnMut(&mut S) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        with_storage_from_context::<S, Q, F, T>(self.wasmer_instance.context_mut(), func)
    }

    /// Requests memory allocation by the instance and returns a pointer
    /// in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, size: usize) -> VmResult<u32> {
        let alloc: Func<u32, u32> = self.func("allocate")?;
        let ptr = alloc.call(to_u32(size)?).context(WasmerRuntimeErr {})?;
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> VmResult<()> {
        let dealloc: Func<u32, ()> = self.func("deallocate")?;
        dealloc.call(ptr).context(WasmerRuntimeErr {})?;
        Ok(())
    }

    /// Copies all data described by the Region at the given pointer from Wasm to the caller.
    pub(crate) fn read_memory(&self, region_ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
        read_region(self.wasmer_instance.context(), region_ptr, max_length)
    }

    /// Copies data to the memory region that was created before using allocate.
    pub(crate) fn write_memory(&mut self, region_ptr: u32, data: &[u8]) -> VmResult<()> {
        write_region(self.wasmer_instance.context(), region_ptr, data)?;
        Ok(())
    }

    pub(crate) fn func<Args, Rets>(&self, name: &str) -> VmResult<Func<Args, Rets, Wasm>>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.wasmer_instance.func(name).context(ResolveErr {})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use wasmer_runtime_core::error::ResolveError;

    use crate::errors::VmError;
    use crate::testing::mock_instance;

    static KIB: usize = 1024;
    static MIB: usize = 1024 * 1024;
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn func_works() {
        let instance = mock_instance(&CONTRACT, &[]);

        // can get func
        let allocate: Func<u32, u32> = instance.func("allocate").expect("error getting func");

        // can call a few times
        let _ptr1 = allocate.call(0).expect("error calling allocate func");
        let _ptr2 = allocate.call(1).expect("error calling allocate func");
        let _ptr3 = allocate.call(33).expect("error calling allocate func");
    }

    #[test]
    fn func_errors_for_non_existent_function() {
        let instance = mock_instance(&CONTRACT, &[]);
        let missing_function = "bar_foo345";
        match instance.func::<(), ()>(missing_function) {
            Err(VmError::ResolveErr { source, .. }) => match source {
                ResolveError::ExportNotFound { name } => assert_eq!(name, missing_function),
                _ => panic!("found unexpected source error"),
            },
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(_) => panic!("must not succeed"),
        }
    }

    #[test]
    fn allocate_deallocate_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        let sizes: Vec<usize> = vec![
            0,
            4,
            40,
            400,
            4 * KIB,
            40 * KIB,
            400 * KIB,
            4 * MIB,
            40 * MIB,
            400 * MIB,
        ];
        for size in sizes.into_iter() {
            let region_ptr = instance.allocate(size).expect("error allocating");
            instance.deallocate(region_ptr).expect("error deallocating");
        }
    }

    #[test]
    fn write_and_read_memory_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        let sizes: Vec<usize> = vec![
            0,
            4,
            40,
            400,
            4 * KIB,
            40 * KIB,
            400 * KIB,
            4 * MIB,
            // disabled for performance reasons, but pass as well
            // 40 * MIB,
            // 400 * MIB,
        ];
        for size in sizes.into_iter() {
            let region_ptr = instance.allocate(size).expect("error allocating");
            let original = vec![170u8; size];
            instance
                .write_memory(region_ptr, &original)
                .expect("error writing");
            let data = instance
                .read_memory(region_ptr, size)
                .expect("error reading");
            assert_eq!(data, original);
            instance.deallocate(region_ptr).expect("error deallocating");
        }
    }

    #[test]
    fn read_memory_errors_when_when_length_is_too_long() {
        let length = 6;
        let max_length = 5;
        let mut instance = mock_instance(&CONTRACT, &[]);

        // Allocate sets length to 0. Write some data to increase length.
        let region_ptr = instance.allocate(length).expect("error allocating");
        let data = vec![170u8; length];
        instance
            .write_memory(region_ptr, &data)
            .expect("error writing");

        match instance.read_memory(region_ptr, max_length) {
            Err(VmError::RegionLengthTooBigErr {
                length, max_length, ..
            }) => {
                assert_eq!(length, 6);
                assert_eq!(max_length, 5);
            }
            Err(err) => panic!("unexpected error: {:?}", err),
            Ok(_) => panic!("must not succeed"),
        };

        instance.deallocate(region_ptr).expect("error deallocating");
    }

    #[test]
    fn get_memory_size_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        assert_eq!(instance.get_memory_size(), 17 * WASM_PAGE_SIZE);

        // 100 KiB require two more pages
        let region_ptr = instance.allocate(100 * 1024).expect("error allocating");

        assert_eq!(instance.get_memory_size(), 19 * WASM_PAGE_SIZE);

        // Deallocating does not shrink memory
        instance.deallocate(region_ptr).expect("error deallocating");
        assert_eq!(instance.get_memory_size(), 19 * WASM_PAGE_SIZE);
    }

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn set_get_and_gas_cranelift_noop() {
        let instance = crate::testing::mock_instance_with_gas_limit(&CONTRACT, &[], 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 1_000_000);
    }

    #[test]
    #[should_panic]
    fn with_context_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let mut instance = mock_instance(&CONTRACT, &[]);
        instance
            .with_storage::<_, ()>(|_store| panic!("trigger failure"))
            .unwrap();
    }
}

#[cfg(test)]
#[cfg(feature = "default-singlepass")]
mod singlepass_test {
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{coins, NativeMsg};

    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::{mock_instance, mock_instance_with_gas_limit};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn set_get_and_gas_singlepass_works() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, &[], 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        let orig_gas = instance.get_gas();

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, NativeMsg>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert_eq!(init_used, 45607);
    }

    #[test]
    fn contract_deducts_gas_handle() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, NativeMsg>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_handle = instance.get_gas();
        let env = mock_env(&instance.api, "verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_handle::<_, _, _, NativeMsg>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        let handle_used = gas_before_handle - instance.get_gas();
        println!("handle used: {}", handle_used);
        assert_eq!(handle_used, 63554);
    }

    #[test]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, &[], 20_000);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init::<_, _, _, NativeMsg>(&mut instance, &env, msg);
        assert!(res.is_err());
    }

    #[test]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init::<_, _, _, NativeMsg>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_query = instance.get_gas();
        // we need to encode the key in base64
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let res = call_query(&mut instance, msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer.as_slice(), b"{\"verifier\":\"verifies\"}");

        let query_used = gas_before_query - instance.get_gas();
        println!("query used: {}", query_used);
        assert_eq!(query_used, 23050);
    }
}
