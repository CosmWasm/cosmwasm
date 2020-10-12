use std::collections::HashSet;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{
    imports, Function, FunctionType, Instance as WasmerInstance, Module, Store, Type, Val,
};
use wasmer_engine_jit::JIT;

use crate::backends::{compile, get_gas_left, set_gas_left};
use crate::context::{
    move_into_context, move_out_of_context, set_storage_readonly, set_wasmer_instance,
    with_querier_from_context, with_storage_from_context, ContextData, Env,
};
use crate::conversion::to_u32;
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::features::required_features_from_wasmer_instance;
use crate::imports::{
    do_canonicalize_address, do_humanize_address, do_query_chain, do_read, do_remove, do_write,
    print_debug_message,
};
#[cfg(feature = "iterator")]
use crate::imports::{do_next, do_scan};
use crate::memory::{read_region, write_region};
use crate::traits::{Api, Extern, Querier, Storage};

const WASM_PAGE_SIZE: u64 = 64 * 1024;

#[derive(Copy, Clone, Debug)]
pub struct GasReport {
    /// The original limit the instance was created with
    pub limit: u64,
    /// The remaining gas that can be spend
    pub remaining: u64,
    /// The amount of gas that was spend and metered externally in operations triggered by this instance
    pub used_externally: u64,
    /// The amount of gas that was spend and metered internally (i.e. by executing Wasm and calling
    /// API methods which are not metered externally)
    pub used_internally: u64,
}

pub struct Instance<S: Storage, A: Api, Q: Querier> {
    /// We put this instance in a box to maintain a constant memory address for the entire
    /// lifetime of the instance in the cache. This is needed e.g. when linking the wasmer
    /// instance to a context. See also https://github.com/CosmWasm/cosmwasm/pull/245
    inner: Box<WasmerInstance>,
    env: Env<S, Q>,
    pub api: A,
    pub required_features: HashSet<String>,
    // This does not store data but only fixes type information
    type_storage: PhantomData<S>,
    type_querier: PhantomData<Q>,
}

impl<S, A, Q> Instance<S, A, Q>
where
    S: Storage + 'static, // 'static is needed here to allow using this in an Env that is cloned into closures
    A: Api + 'static,     // 'static is needed here to allow copying API instances into closures
    Q: Querier + 'static, // 'static is needed here to allow using this in an Env that is cloned into closures
{
    /// This is the only Instance constructor that can be called from outside of cosmwasm-vm,
    /// e.g. in test code that needs a customized variant of cosmwasm_vm::testing::mock_instance*.
    pub fn from_code(
        code: &[u8],
        deps: Extern<S, A, Q>,
        gas_limit: u64,
        print_debug: bool,
    ) -> VmResult<Self> {
        let module = compile(code)?;
        Instance::from_module(&module, deps, gas_limit, print_debug)
    }

    pub(crate) fn from_module(
        module: &Module,
        deps: Extern<S, A, Q>,
        gas_limit: u64,
        print_debug: bool,
    ) -> VmResult<Self> {
        // copy this so it can be moved into the closures, without pulling in deps
        let api = deps.api;

        let engine = JIT::headless().engine();
        let store = Store::new(&engine);

        let mut env = Env {
            memory: wasmer::Memory::new(&store, wasmer::MemoryType::new(0, Some(5000), false))
                .expect("could not create memory"),
            context_data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
        };
        let _env2 = env.clone();
        let _env3 = env.clone();

        let i32_to_i32 = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let i32_to_void = FunctionType::new(vec![Type::I32], vec![]);
        let i32i32_to_void = FunctionType::new(vec![Type::I32, Type::I32], vec![]);
        let i32i32_to_i32 = FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32]);
        let i32i32i32_to_i32 =
            FunctionType::new(vec![Type::I32, Type::I32, Type::I32], vec![Type::I32]);

        let import_obj = imports! {
            "env" => {
                // Reads the database entry at the given key into the the value.
                // Returns 0 if key does not exist and pointer to result region otherwise.
                // Ownership of the key pointer is not transferred to the host.
                // Ownership of the value pointer is transferred to the contract.
                "db_read" => Function::new_with_env(&store, &i32_to_i32, env.clone(), |mut env, args| {
                    let key_ptr = args[0].unwrap_i32() as u32;
                    let ptr = do_read::<S, Q>(&mut env, key_ptr)?;
                    Ok(vec![ptr.into()])
                }),
                // Writes the given value into the database entry at the given key.
                // Ownership of both input and output pointer is not transferred to the host.
                "db_write" => Function::new_with_env(&store, &i32i32_to_void, env.clone(), |mut env, args| {
                    let key_ptr = args[0].unwrap_i32() as u32;
                    let value_ptr = args[1].unwrap_i32() as u32;
                    do_write::<S, Q>(&mut env, key_ptr, value_ptr)?;
                    Ok(vec![])
                }),
                // Removes the value at the given key. Different than writing &[] as future
                // scans will not find this key.
                // At the moment it is not possible to differentiate between a key that existed before and one that did not exist (https://github.com/CosmWasm/cosmwasm/issues/290).
                // Ownership of both key pointer is not transferred to the host.
                "db_remove" => Function::new_with_env(&store, &i32_to_void, env.clone(), |mut env, args| {
                    let key_ptr = args[0].unwrap_i32() as u32;
                    do_remove::<S, Q>(&mut env, key_ptr)?;
                    Ok(vec![])
                }),
                // Reads human address from source_ptr and writes canonicalized representation to destination_ptr.
                // A prepared and sufficiently large memory Region is expected at destination_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns a non-zero memory location to a Region containing an UTF-8 encoded error string for invalid inputs.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Function::new_with_env(&store, &i32i32_to_i32, env.clone(), move |mut env, args| {
                    let source_ptr = args[0].unwrap_i32() as u32;
                    let destination_ptr = args[1].unwrap_i32() as u32;
                    let ptr = do_canonicalize_address::<A, S, Q>(api, &mut env, source_ptr, destination_ptr)?;
                    Ok(vec![ptr.into()])
                }),
                // Reads canonical address from source_ptr and writes humanized representation to destination_ptr.
                // A prepared and sufficiently large memory Region is expected at destination_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns a non-zero memory location to a Region containing an UTF-8 encoded error string for invalid inputs.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Function::new_with_env(&store, &i32i32_to_i32, env.clone(), move |mut env, args| {
                    let source_ptr = args[0].unwrap_i32() as u32;
                    let destination_ptr = args[1].unwrap_i32() as u32;
                    let ptr = do_humanize_address::<A, S, Q>(api, &mut env, source_ptr, destination_ptr)?;
                    Ok(vec![ptr.into()])
                }),
                // Allows the contract to emit debug logs that the host can either process or ignore.
                // This is never written to chain.
                // Takes a pointer argument of a memory region that must contain an UTF-8 encoded string.
                // Ownership of both input and output pointer is not transferred to the host.
                "debug" => Function::new_with_env(&store, &i32_to_void, env.clone(), move |mut env, args| {
                    let message_ptr = args[0].unwrap_i32() as u32;
                    if print_debug {
                        print_debug_message(&mut env, message_ptr)?;
                    }
                    Ok(vec![])
                }),
                "query_chain" => Function::new_with_env(&store, &i32_to_i32, env.clone(), |mut env, args| {
                    let request_ptr = args[0].unwrap_i32() as u32;
                    let response_ptr = do_query_chain::<S, Q>(&mut env, request_ptr)?;
                    Ok(vec![response_ptr.into()])
                }),
                // Creates an iterator that will go from start to end.
                // If start_ptr == 0, the start is unbounded.
                // If end_ptr == 0, the end is unbounded.
                // Order is defined in cosmwasm_std::Order and may be 1 (ascending) or 2 (descending). All other values result in an error.
                // Ownership of both start and end pointer is not transferred to the host.
                // Returns an iterator ID.
                "db_scan" => Function::new_with_env(&store, &i32i32i32_to_i32, env.clone(), |mut env, args| {
                    let start_ptr = args[0].unwrap_i32() as u32;
                    let end_ptr = args[1].unwrap_i32() as u32;
                    let order = args[2].unwrap_i32();
                    #[cfg(feature = "iterator")]
                    {
                        let response_ptr = do_scan::<S, Q>(&mut env, start_ptr, end_ptr, order)?;
                        Ok(vec![response_ptr.into()])
                    }
                    #[cfg(not(feature = "iterator"))]
                    {
                        Err(wasmer_engine::RuntimeError::new("Import db_scan not implemented due to missing iterator feature"))
                    }
                }),
                // Get next element of iterator with ID `iterator_id`.
                // Creates a region containing both key and value and returns its address.
                // Ownership of the result region is transferred to the contract.
                // The KV region uses the format value || key || keylen, where keylen is a fixed size big endian u32 value.
                // An empty key (i.e. KV region ends with \0\0\0\0) means no more element, no matter what the value is.
                "db_next" => Function::new_with_env(&store, &i32_to_i32, env.clone(), |mut env, args| {
                    #[cfg(feature = "iterator")]
                    {
                        let iterator_id = args[0].unwrap_i32() as u32;
                        let response_ptr = do_next::<S, Q>(&mut env, iterator_id)?;
                        Ok(vec![response_ptr.into()])
                    }
                    #[cfg(not(feature = "iterator"))]
                    {
                        Err(wasmer_engine::RuntimeError::new("Import db_next not implemented due to missing iterator feature"))
                    }
                }),
            },
        };

        let wasmer_instance = Box::from(WasmerInstance::new(&module, &import_obj).map_err(
            |original| {
                VmError::instantiation_err(format!("Error instantiating module: {:?}", original))
            },
        )?);

        set_gas_left(&mut env, gas_limit);
        env.with_gas_state_mut(|gas_state| {
            gas_state.set_gas_limit(gas_limit);
        });
        let required_features = required_features_from_wasmer_instance(wasmer_instance.as_ref());
        let instance_ptr = NonNull::from(wasmer_instance.as_ref());
        set_wasmer_instance::<S, Q>(&mut env, Some(instance_ptr));
        move_into_context(&mut env, deps.storage, deps.querier);
        let instance = Instance {
            inner: wasmer_instance,
            env,
            api: deps.api,
            required_features,
            type_storage: PhantomData::<S> {},
            type_querier: PhantomData::<Q> {},
        };
        Ok(instance)
    }

    /// Decomposes this instance into its components.
    /// External dependencies are returned for reuse, the rest is dropped.
    pub fn recycle(mut self) -> Option<Extern<S, A, Q>> {
        if let (Some(storage), Some(querier)) = move_out_of_context(&mut self.env) {
            Some(Extern {
                storage,
                api: self.api,
                querier,
            })
        } else {
            None
        }
    }

    /// Returns the size of the default memory in bytes.
    /// This provides a rough idea of the peak memory consumption. Note that
    /// Wasm memory always grows in 64 KiB steps (pages) and can never shrink
    /// (https://github.com/WebAssembly/design/issues/1300#issuecomment-573867836).
    pub fn get_memory_size(&self) -> u64 {
        self.env.memory.data_size()
    }

    /// Returns the currently remaining gas.
    pub fn get_gas_left(&self) -> u64 {
        self.create_gas_report().remaining
    }

    /// Creates and returns a gas report.
    /// This is a snapshot and multiple reports can be created during the lifetime of
    /// an instance.
    pub fn create_gas_report(&self) -> GasReport {
        let state = self.env.with_gas_state(|gas_state| gas_state.clone());
        let gas_left = get_gas_left(&self.env);
        GasReport {
            limit: state.gas_limit,
            remaining: gas_left,
            used_externally: state.externally_used_gas,
            used_internally: state.get_gas_used_in_wasmer(gas_left),
        }
    }

    /// Sets the readonly storage flag on this instance. Since one instance can be used
    /// for multiple calls in integration tests, this should be set to the desired value
    /// right before every call.
    pub fn set_storage_readonly(&mut self, new_value: bool) {
        set_storage_readonly::<S, Q>(&mut self.env, new_value);
    }

    pub fn with_storage<F: FnOnce(&mut S) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        with_storage_from_context::<S, Q, F, T>(self.env.clone(), func)
    }

    pub fn with_querier<F: FnOnce(&mut Q) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        with_querier_from_context::<S, Q, F, T>(self.env.clone(), func)
    }

    /// Requests memory allocation by the instance and returns a pointer
    /// in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, size: usize) -> VmResult<u32> {
        let ret = self.call_function("allocate", &[to_u32(size)?.into()])?;
        let ptr = ret.as_ref()[0].unwrap_i32() as u32;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> VmResult<()> {
        self.call_function("deallocate", &[ptr.into()])?;
        Ok(())
    }

    /// Copies all data described by the Region at the given pointer from Wasm to the caller.
    pub(crate) fn read_memory(&self, region_ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
        read_region(&self.env.memory, region_ptr, max_length)
    }

    /// Copies data to the memory region that was created before using allocate.
    pub(crate) fn write_memory(&mut self, region_ptr: u32, data: &[u8]) -> VmResult<()> {
        write_region(&self.env.memory, region_ptr, data)?;
        Ok(())
    }

    pub(crate) fn call_function(&self, name: &str, args: &[Val]) -> VmResult<Box<[Val]>> {
        let function = self.inner.exports.get_function(name)?;
        let result = function.call(args)?;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::context::is_storage_readonly;
    use crate::errors::VmError;
    use crate::testing::{
        mock_dependencies, mock_env, mock_info, mock_instance, mock_instance_with_balances,
        mock_instance_with_failing_api, mock_instance_with_gas_limit, MockQuerier, MockStorage,
    };
    use crate::traits::Storage;
    use crate::{call_init, FfiError};
    use cosmwasm_std::{
        coin, coins, from_binary, AllBalanceResponse, BalanceResponse, BankQuery, Empty, HumanAddr,
        QueryRequest,
    };
    use wabt::wat2wasm;

    const KIB: usize = 1024;
    const MIB: usize = 1024 * 1024;
    const DEFAULT_GAS_LIMIT: u64 = 500_000;
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MS = MockStorage;
    type MQ = MockQuerier;

    #[test]
    fn required_features_works() {
        let deps = mock_dependencies(&[]);
        let instance = Instance::from_code(CONTRACT, deps, DEFAULT_GAS_LIMIT, false).unwrap();
        assert_eq!(instance.required_features.len(), 0);
    }

    #[test]
    fn required_features_works_for_many_exports() {
        let wasm = wat2wasm(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
            )"#,
        )
        .unwrap();

        let deps = mock_dependencies(&[]);
        let instance = Instance::from_code(&wasm, deps, DEFAULT_GAS_LIMIT, false).unwrap();
        assert_eq!(instance.required_features.len(), 3);
        assert!(instance.required_features.contains("nutrients"));
        assert!(instance.required_features.contains("sun"));
        assert!(instance.required_features.contains("water"));
    }

    #[test]
    fn call_func_works() {
        let instance = mock_instance(&CONTRACT, &[]);

        // can call function few times
        let _ptr1 = instance
            .call_function("allocate", &[0u32.into()])
            .expect("error calling allocate");
        let _ptr2 = instance
            .call_function("allocate", &[1u32.into()])
            .expect("error calling allocate");
        let _ptr3 = instance
            .call_function("allocate", &[33u32.into()])
            .expect("error calling allocate");
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
    fn errors_in_imports_are_unwrapped_from_wasmer_errors() {
        // set up an instance that will experience an error in an import
        let error_message = "Api failed intentionally";
        let mut instance = mock_instance_with_failing_api(&CONTRACT, &[], error_message);
        let init_result = call_init::<_, _, _, serde_json::Value>(
            &mut instance,
            &mock_env(),
            &mock_info("someone", &[]),
            b"{\"verifier\": \"some1\", \"beneficiary\": \"some2\"}",
        );

        // in this case we get a `VmError::FfiError` rather than a `VmError::RuntimeErr` because the conversion
        // from wasmer `RuntimeError` to `VmError` unwraps errors that happen in WASM imports.
        match init_result.unwrap_err() {
            VmError::FfiErr {
                source: FfiError::Unknown { msg, .. },
            } if msg == Some(error_message.to_string()) => {}
            other => panic!("unexpected error: {:?}", other),
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

        let result = instance.read_memory(region_ptr, max_length);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 6);
                assert_eq!(max_length, 5);
            }
            err => panic!("unexpected error: {:?}", err),
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
    fn set_get_and_gas_cranelift() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas_left();
        assert_eq!(orig_gas, 1_000_000); // We expect a dummy value for cranelift
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn set_get_and_gas_singlepass() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas_left();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    #[cfg(feature = "default-cranelift")]
    fn create_gas_report_works_cranelift() {
        const LIMIT: u64 = 7_000_000;
        /// Value hardcoded in cranelift backend
        const FAKE_REMANING: u64 = 1_000_000;
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, LIMIT);

        let report1 = instance.create_gas_report();
        assert_eq!(report1.used_externally, 0);
        assert_eq!(report1.used_internally, LIMIT - FAKE_REMANING);
        assert_eq!(report1.limit, LIMIT);
        assert_eq!(report1.remaining, FAKE_REMANING);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let report2 = instance.create_gas_report();
        assert_eq!(report2.used_externally, 0);
        assert_eq!(report2.used_internally, LIMIT - FAKE_REMANING);
        assert_eq!(report2.limit, LIMIT);
        assert_eq!(report2.remaining, FAKE_REMANING);
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn create_gas_report_works_singlepass() {
        const LIMIT: u64 = 7_000_000;
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, LIMIT);

        let report1 = instance.create_gas_report();
        assert_eq!(report1.used_externally, 0);
        assert_eq!(report1.used_internally, 0);
        assert_eq!(report1.limit, LIMIT);
        assert_eq!(report1.remaining, LIMIT);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let report2 = instance.create_gas_report();
        assert_eq!(report2.used_externally, 146);
        assert_eq!(report2.used_internally, 67203);
        assert_eq!(report2.limit, LIMIT);
        assert_eq!(
            report2.remaining,
            LIMIT - report2.used_externally - report2.used_internally
        );
    }

    /*
    TODO: Move to env testing
    #[test]
    fn set_storage_readonly_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        assert_eq!(
            is_storage_readonly::<MS, MQ>(&instance.inner.context()),
            true
        );

        instance.set_storage_readonly(false);
        assert_eq!(
            is_storage_readonly::<MS, MQ>(&instance.inner.context()),
            false
        );

        instance.set_storage_readonly(false);
        assert_eq!(
            is_storage_readonly::<MS, MQ>(&instance.inner.context()),
            false
        );

        instance.set_storage_readonly(true);
        assert_eq!(
            is_storage_readonly::<MS, MQ>(&instance.inner.context()),
            true
        );
    }
    */

    #[test]
    fn with_storage_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // initial check
        instance
            .with_storage(|store| {
                assert!(store.get(b"foo").0.unwrap().is_none());
                Ok(())
            })
            .unwrap();

        // write some data
        instance
            .with_storage(|store| {
                store.set(b"foo", b"bar").0.unwrap();
                Ok(())
            })
            .unwrap();

        // read some data
        instance
            .with_storage(|store| {
                assert_eq!(store.get(b"foo").0.unwrap(), Some(b"bar".to_vec()));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn with_storage_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let mut instance = mock_instance(&CONTRACT, &[]);
        instance
            .with_storage::<_, ()>(|_store| panic!("trigger failure"))
            .unwrap();
    }

    #[test]
    fn with_querier_works_readonly() {
        let rich_addr = HumanAddr::from("foobar");
        let rich_balance = vec![coin(10000, "gold"), coin(8000, "silver")];
        let mut instance = mock_instance_with_balances(&CONTRACT, &[(&rich_addr, &rich_balance)]);

        // query one
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 8000);
                assert_eq!(amount.denom, "silver");
                Ok(())
            })
            .unwrap();

        // query all
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::AllBalances {
                            address: rich_addr.clone(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let AllBalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.len(), 2);
                assert_eq!(amount[0].amount.u128(), 10000);
                assert_eq!(amount[0].denom, "gold");
                assert_eq!(amount[1].amount.u128(), 8000);
                assert_eq!(amount[1].denom, "silver");

                Ok(())
            })
            .unwrap();
    }

    /// This is needed for writing intagration tests in which the balance of a contract changes over time
    #[test]
    fn with_querier_allows_updating_balances() {
        let rich_addr = HumanAddr::from("foobar");
        let rich_balance1 = vec![coin(10000, "gold"), coin(500, "silver")];
        let rich_balance2 = vec![coin(10000, "gold"), coin(8000, "silver")];
        let mut instance = mock_instance_with_balances(&CONTRACT, &[(&rich_addr, &rich_balance1)]);

        // Get initial state
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 500);
                Ok(())
            })
            .unwrap();

        // Update balance
        instance
            .with_querier(|querier| {
                querier.update_balance(&rich_addr, rich_balance2);
                Ok(())
            })
            .unwrap();

        // Get updated state
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 8000);
                Ok(())
            })
            .unwrap();
    }
}

#[cfg(test)]
#[cfg(feature = "default-singlepass")]
mod singlepass_test {
    use cosmwasm_std::{coins, Empty};

    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::{mock_env, mock_info, mock_instance, mock_instance_with_gas_limit};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        let orig_gas = instance.get_gas_left();

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let init_used = orig_gas - instance.get_gas_left();
        assert_eq!(init_used, 67349);
    }

    #[test]
    fn contract_deducts_gas_handle() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_handle = instance.get_gas_left();
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_handle::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let handle_used = gas_before_handle - instance.get_gas_left();
        assert_eq!(handle_used, 196538);
    }

    #[test]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, 20_000);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_query = instance.get_gas_left();
        // we need to encode the key in base64
        let msg = r#"{"verifier":{}}"#.as_bytes();
        let res = call_query(&mut instance, &mock_env(), msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer.as_slice(), b"{\"verifier\":\"verifies\"}");

        let query_used = gas_before_query - instance.get_gas_left();
        assert_eq!(query_used, 54076);
    }
}
