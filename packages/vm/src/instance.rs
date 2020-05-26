use std::collections::HashSet;
use std::marker::PhantomData;

pub use wasmer_runtime_core::typed_func::Func;
use wasmer_runtime_core::{
    imports,
    module::Module,
    typed_func::{Wasm, WasmTypeList},
    vm::Ctx,
};

use crate::backends::{compile, get_gas, set_gas};
use crate::context::{
    move_into_context, move_out_of_context, setup_context, with_querier_from_context,
    with_storage_from_context,
};
use crate::conversion::to_u32;
use crate::errors::{make_instantiation_err, VmResult};
use crate::features::required_features_from_wasmer_instance;
use crate::imports::{
    do_canonicalize_address, do_humanize_address, do_query_chain, do_read, do_remove, do_write,
};
#[cfg(feature = "iterator")]
use crate::imports::{do_next, do_scan};
use crate::memory::{get_memory_info, read_region, write_region};
use crate::traits::{Api, Extern, Querier, Storage};

static WASM_PAGE_SIZE: u64 = 64 * 1024;

pub struct Instance<S: Storage + 'static, A: Api + 'static, Q: Querier + 'static> {
    wasmer_instance: wasmer_runtime_core::instance::Instance,
    pub api: A,
    pub required_features: HashSet<String>,
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
    /// This is the only Instance constructor that can be called from outside of cosmwasm-vm,
    /// e.g. in test code that needs a customized variant of cosmwasm_vm::testing::mock_instance*.
    pub fn from_code(code: &[u8], deps: Extern<S, A, Q>, gas_limit: u64) -> VmResult<Self> {
        let module = compile(code)?;
        Instance::from_module(&module, deps, gas_limit)
    }

    pub(crate) fn from_module(
        module: &Module,
        deps: Extern<S, A, Q>,
        gas_limit: u64,
    ) -> VmResult<Self> {
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
                "db_read" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> VmResult<i32> {
                    do_read::<S, Q>(ctx, key_ptr, value_ptr)
                }),
                // Writes the given value into the database entry at the given key.
                // Ownership of both input and output pointer is not transferred to the host.
                // Returns 0 on success. Returns negative value on error.
                "db_write" => Func::new(move |ctx: &mut Ctx, key_ptr: u32, value_ptr: u32| -> VmResult<i32> {
                    do_write::<S, Q>(ctx, key_ptr, value_ptr)
                }),
                // Removes the value at the given key. Different than writing &[] as future
                // scans will not find this key.
                // At the moment it is not possible to differentiate between a key that existed before and one that did not exist (https://github.com/CosmWasm/cosmwasm/issues/290).
                // Ownership of both key pointer is not transferred to the host.
                // Returns 0 on success. Returns negative value on error.
                "db_remove" => Func::new(move |ctx: &mut Ctx, key_ptr: u32| -> VmResult<i32> {
                    do_remove::<S, Q>(ctx, key_ptr)
                }),
                // Reads human address from human_ptr and writes canonicalized representation to canonical_ptr.
                // A prepared and sufficiently large memory Region is expected at canonical_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "canonicalize_address" => Func::new(move |ctx: &mut Ctx, human_ptr: u32, canonical_ptr: u32| -> VmResult<i32> {
                    do_canonicalize_address(api, ctx, human_ptr, canonical_ptr)
                }),
                // Reads canonical address from canonical_ptr and writes humanized representation to human_ptr.
                // A prepared and sufficiently large memory Region is expected at human_ptr that points to pre-allocated memory.
                // Returns 0 on success. Returns negative value on error.
                // Ownership of both input and output pointer is not transferred to the host.
                "humanize_address" => Func::new(move |ctx: &mut Ctx, canonical_ptr: u32, human_ptr: u32| -> VmResult<i32> {
                    do_humanize_address(api, ctx, canonical_ptr, human_ptr)
                }),
                "query_chain" => Func::new(move |ctx: &mut Ctx, request_ptr: u32, response_ptr: u32| -> VmResult<i32> {
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
                "db_scan" => Func::new(move |ctx: &mut Ctx, start_ptr: u32, end_ptr: u32, order: i32| -> VmResult<i32> {
                    do_scan::<S, Q>(ctx, start_ptr, end_ptr, order)
                }),
                // Get next element of iterator with ID `iterator_id`.
                // Expects Regions in key_ptr and value_ptr, in which the result is written.
                // An empty key value (Region of length 0) means no more element.
                // Ownership of both key and value pointer is not transferred to the host.
                "db_next" => Func::new(move |ctx: &mut Ctx, iterator_id: u32, key_ptr: u32, value_ptr: u32| -> VmResult<i32> {
                    do_next::<S, Q>(ctx, iterator_id, key_ptr, value_ptr)
                }),
            },
        });

        let wasmer_instance = module.instantiate(&import_obj).map_err(|original| {
            make_instantiation_err(format!("Error instantiating module: {:?}", original))
        })?;
        Ok(Instance::from_wasmer(wasmer_instance, deps, gas_limit))
    }

    pub(crate) fn from_wasmer(
        mut wasmer_instance: wasmer_runtime_core::Instance,
        deps: Extern<S, A, Q>,
        gas_limit: u64,
    ) -> Self {
        set_gas(&mut wasmer_instance, gas_limit);
        let required_features = required_features_from_wasmer_instance(&wasmer_instance);
        move_into_context(wasmer_instance.context_mut(), deps.storage, deps.querier);
        Instance {
            wasmer_instance,
            api: deps.api,
            required_features,
            type_storage: PhantomData::<S> {},
            type_querier: PhantomData::<Q> {},
        }
    }

    /// Takes ownership of instance and decomposes it into its components.
    /// The components we want to preserve are returned, the rest is dropped.
    pub(crate) fn recycle(
        mut instance: Self,
    ) -> (wasmer_runtime_core::Instance, Option<Extern<S, A, Q>>) {
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

    pub fn with_storage<F: FnOnce(&mut S) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        with_storage_from_context::<S, Q, F, T>(self.wasmer_instance.context_mut(), func)
    }

    pub fn with_querier<F: FnOnce(&mut Q) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        with_querier_from_context::<S, Q, F, T>(self.wasmer_instance.context_mut(), func)
    }

    /// Requests memory allocation by the instance and returns a pointer
    /// in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, size: usize) -> VmResult<u32> {
        let alloc: Func<u32, u32> = self.func("allocate")?;
        let ptr = alloc.call(to_u32(size)?)?;
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> VmResult<()> {
        let dealloc: Func<u32, ()> = self.func("deallocate")?;
        dealloc.call(ptr)?;
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
        let function = self.wasmer_instance.exports.get(name)?;
        Ok(function)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::VmError;
    use crate::testing::{
        mock_dependencies, mock_env, mock_instance, mock_instance_with_balances,
        mock_instance_with_failing_api, mock_instance_with_gas_limit, MockApi, MOCK_CONTRACT_ADDR,
    };
    use crate::traits::ReadonlyStorage;
    use crate::{call_init, FfiError};
    use cosmwasm_std::{
        coin, from_binary, AllBalanceResponse, BalanceResponse, BankQuery, HumanAddr, Never,
        QueryRequest,
    };
    use wabt::wat2wasm;

    static KIB: usize = 1024;
    static MIB: usize = 1024 * 1024;
    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");
    static REFLECT_CONTRACT: &[u8] = include_bytes!("../../../contracts/reflect/contract.wasm");
    static DEFAULT_GAS_LIMIT: u64 = 500_000;

    #[test]
    fn required_features_works() {
        let deps = mock_dependencies(20, &[]);
        let instance = Instance::from_code(CONTRACT, deps, DEFAULT_GAS_LIMIT).unwrap();
        assert_eq!(instance.required_features.len(), 0);

        let deps = mock_dependencies(20, &[]);
        let instance = Instance::from_code(REFLECT_CONTRACT, deps, DEFAULT_GAS_LIMIT).unwrap();
        assert_eq!(instance.required_features.len(), 1);
        assert!(instance.required_features.contains("staking"));
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

        let deps = mock_dependencies(20, &[]);
        let instance = Instance::from_code(&wasm, deps, DEFAULT_GAS_LIMIT).unwrap();
        assert_eq!(instance.required_features.len(), 3);
        assert!(instance.required_features.contains("nutrients"));
        assert!(instance.required_features.contains("sun"));
        assert!(instance.required_features.contains("water"));
    }

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
        match instance.func::<(), ()>(missing_function).err().unwrap() {
            VmError::ResolveErr { msg, .. } => assert_eq!(
                msg,
                "Wasmer resolve error: ExportNotFound { name: \"bar_foo345\" }"
            ),
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn func_errors_for_wrong_signature() {
        let instance = mock_instance(&CONTRACT, &[]);
        match instance.func::<(), ()>("allocate").err().unwrap() {
            VmError::ResolveErr { msg, .. } => assert_eq!(
                msg,
                "Wasmer resolve error: Signature { expected: FuncSig { params: [I32], returns: [I32] }, found: [] }"
            ),
            e => panic!("unexpected error: {:?}", e),
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
    fn errors_in_imports_are_unwrapped_from_wasmer_errors() {
        // set up an instance that will experience an error in an import
        let mut instance = mock_instance_with_failing_api(&CONTRACT, &[]);
        let init_result = call_init::<_, _, _, serde_json::Value>(
            &mut instance,
            &mock_env(&MockApi::new(MOCK_CONTRACT_ADDR.len()), "someone", &[]),
            b"{\"verifier\": \"some1\", \"beneficiary\": \"some2\"}",
        );

        // in this case we get a `VmError::FfiError` rather than a `VmError::RuntimeErr` because the conversion
        // from wasmer `RuntimeError` to `VmError` unwraps errors that happen in WASM imports.
        match init_result.unwrap_err() {
            VmError::FfiErr {
                source: FfiError::Other { error, .. },
            } if error == "canonical_address failed intentionally" => {}
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

        match instance.read_memory(region_ptr, max_length) {
            Err(VmError::RegionLengthTooBig {
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
    fn set_get_and_gas_cranelift() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 1_000_000); // We expect a dummy value for cranelift
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn set_get_and_gas_singlepass() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    fn with_storage_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // initial check
        instance
            .with_storage(|store| {
                assert!(store.get(b"foo").unwrap().is_none());
                Ok(())
            })
            .unwrap();

        // write some data
        instance
            .with_storage(|store| {
                store.set(b"foo", b"bar").unwrap();
                Ok(())
            })
            .unwrap();

        // read some data
        instance
            .with_storage(|store| {
                assert_eq!(store.get(b"foo").unwrap(), Some(b"bar".to_vec()));
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
                    .handle_query::<Never>(&QueryRequest::Bank(BankQuery::Balance {
                        address: rich_addr.clone(),
                        denom: "silver".to_string(),
                    }))?
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
                    .handle_query::<Never>(&QueryRequest::Bank(BankQuery::AllBalances {
                        address: rich_addr.clone(),
                    }))?
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
                    .handle_query::<Never>(&QueryRequest::Bank(BankQuery::Balance {
                        address: rich_addr.clone(),
                        denom: "silver".to_string(),
                    }))?
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
                    .handle_query::<Never>(&QueryRequest::Bank(BankQuery::Balance {
                        address: rich_addr.clone(),
                        denom: "silver".to_string(),
                    }))?
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
    use cosmwasm_std::{coins, Never};

    use crate::calls::{call_handle, call_init, call_query};
    use crate::testing::{mock_env, mock_instance, mock_instance_with_gas_limit};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    #[test]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        let orig_gas = instance.get_gas();

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Never>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        let init_used = orig_gas - instance.get_gas();
        println!("init used: {}", init_used);
        assert_eq!(init_used, 65257);
    }

    #[test]
    fn contract_deducts_gas_handle() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        call_init::<_, _, _, Never>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_handle = instance.get_gas();
        let env = mock_env(&instance.api, "verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_handle::<_, _, _, Never>(&mut instance, &env, msg)
            .unwrap()
            .unwrap();

        let handle_used = gas_before_handle - instance.get_gas();
        println!("handle used: {}", handle_used);
        assert_eq!(handle_used, 95374);
    }

    #[test]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, 20_000);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let res = call_init::<_, _, _, Never>(&mut instance, &env, msg);
        assert!(res.is_err());
    }

    #[test]
    fn query_works_with_metering() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let env = mock_env(&instance.api, "creator", &coins(1000, "earth"));
        let msg = r#"{"verifier": "verifies", "beneficiary": "benefits"}"#.as_bytes();
        let _res = call_init::<_, _, _, Never>(&mut instance, &env, msg)
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
        assert_eq!(query_used, 32750);
    }
}
