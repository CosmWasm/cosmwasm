//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{HostEnvInitError, Instance as WasmerInstance, Memory, Val, WasmerEnv};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

use crate::backend::{Api, GasInfo, Querier, Storage};
use crate::errors::{VmError, VmResult};

#[derive(Debug)]
pub struct InsufficientGasLeft;

/** context data **/

#[derive(Clone, PartialEq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation.
    pub gas_limit: u64,
    /// Tracking the gas used in the cosmos SDK, in cosmwasm units.
    #[allow(unused)]
    pub externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }

    #[allow(unused)]
    fn increase_externally_used_gas(&mut self, amount: u64) {
        self.externally_used_gas += amount;
    }

    pub(crate) fn set_gas_limit(&mut self, gas_limit: u64) {
        self.gas_limit = gas_limit;
    }

    /// Get the amount of gas units still left for the rest of the calculation.
    ///
    /// We need the amount of gas used in wasmer since it is not tracked inside this object.
    #[allow(unused)]
    fn get_gas_left(&self, wasmer_used_gas: u64) -> u64 {
        self.gas_limit
            .saturating_sub(self.externally_used_gas)
            .saturating_sub(wasmer_used_gas)
    }

    /// Get the amount of gas units used so far inside wasmer.
    ///
    /// We need the amount of gas left in wasmer since it is not tracked inside this object.
    #[allow(unused)]
    pub(crate) fn get_gas_used_in_wasmer(&self, wasmer_gas_left: u64) -> u64 {
        self.gas_limit
            .saturating_sub(self.externally_used_gas)
            .saturating_sub(wasmer_gas_left)
    }
}

/// A environment that provides access to the ContextData.
/// The environment is clonable but clones access the same underlying data.
pub struct Environment<A: Api, S: Storage, Q: Querier> {
    pub api: A,
    pub print_debug: bool,
    data: Arc<RwLock<ContextData<S, Q>>>,
}

unsafe impl<A: Api, S: Storage, Q: Querier> Send for Environment<A, S, Q> {}

unsafe impl<A: Api, S: Storage, Q: Querier> Sync for Environment<A, S, Q> {}

impl<A: Api, S: Storage, Q: Querier> Clone for Environment<A, S, Q> {
    fn clone(&self) -> Self {
        Environment {
            api: self.api,
            print_debug: self.print_debug,
            data: self.data.clone(),
        }
    }
}

impl<A: Api, S: Storage, Q: Querier> WasmerEnv for Environment<A, S, Q> {
    fn init_with_instance(&mut self, _instance: &WasmerInstance) -> Result<(), HostEnvInitError> {
        Ok(())
    }
}

impl<A: Api, S: Storage, Q: Querier> Environment<A, S, Q> {
    pub fn new(api: A, gas_limit: u64, print_debug: bool) -> Self {
        Environment {
            api,
            print_debug,
            data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
        }
    }

    pub fn with_context_data_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut ContextData<S, Q>) -> R,
    {
        let mut guard = self.data.as_ref().write().unwrap();
        let context_data = guard.borrow_mut();
        callback(context_data)
    }

    pub fn with_context_data<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&ContextData<S, Q>) -> R,
    {
        let guard = self.data.as_ref().read().unwrap();
        let context_data = guard.borrow();
        callback(context_data)
    }

    pub fn with_gas_state_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut GasState) -> R,
    {
        self.with_context_data_mut(|context_data| callback(&mut context_data.gas_state))
    }

    pub fn with_gas_state<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&GasState) -> R,
    {
        self.with_context_data(|context_data| callback(&context_data.gas_state))
    }

    /// Calls a function with the given name and arguments.
    /// The number of return values is variable and controlled by the guest.
    /// Usually we expect 0 or 1 return values. Use [`Self::call_function0`]
    /// or [`Self::call_function1`] to ensure the number of return values is checked.
    fn call_function(&self, name: &str, args: &[Val]) -> VmResult<Box<[Val]>> {
        // Clone function before calling it to avoid dead locks
        let func = self.with_context_data(|context_data| match context_data.wasmer_instance {
            Some(instance_ptr) => {
                let func = unsafe { instance_ptr.as_ref() }
                    .exports
                    .get_function(name)?;
                Ok(func.clone())
            }
            None => Err(VmError::uninitialized_context_data("wasmer_instance")),
        })?;

        func.call(args).map_err(|runtime_err| -> VmError {
            self.with_context_data(|context_data| match context_data.wasmer_instance {
                Some(instance_ptr) => {
                    let instance_ref = unsafe { instance_ptr.as_ref() };
                    match get_remaining_points(instance_ref) {
                        MeteringPoints::Remaining(_) => VmError::from(runtime_err),
                        MeteringPoints::Exhausted => VmError::gas_depletion(),
                    }
                }
                None => VmError::uninitialized_context_data("wasmer_instance"),
            })
        })
    }

    pub fn call_function0(&self, name: &str, args: &[Val]) -> VmResult<()> {
        let result = self.call_function(name, args)?;
        let expected = 0;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(())
    }

    pub fn call_function1(&self, name: &str, args: &[Val]) -> VmResult<Val> {
        let result = self.call_function(name, args)?;
        let expected = 1;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(result[0].clone())
    }

    pub fn with_storage_from_context<C, T>(&self, callback: C) -> VmResult<T>
    where
        C: FnOnce(&mut S) -> VmResult<T>,
    {
        self.with_context_data_mut(|context_data| match context_data.storage.as_mut() {
            Some(data) => callback(data),
            None => Err(VmError::uninitialized_context_data("storage")),
        })
    }

    pub fn with_querier_from_context<C, T>(&self, callback: C) -> VmResult<T>
    where
        C: FnOnce(&mut Q) -> VmResult<T>,
    {
        self.with_context_data_mut(|context_data| match context_data.querier.as_mut() {
            Some(querier) => callback(querier),
            None => Err(VmError::uninitialized_context_data("querier")),
        })
    }

    /// Creates a back reference from a contact to its partent instance
    pub fn set_wasmer_instance(&self, wasmer_instance: Option<NonNull<WasmerInstance>>) {
        self.with_context_data_mut(|context_data| {
            context_data.wasmer_instance = wasmer_instance;
        });
    }

    /// Returns true iff the storage is set to readonly mode
    pub fn is_storage_readonly(&self) -> bool {
        self.with_context_data(|context_data| context_data.storage_readonly)
    }

    pub fn set_storage_readonly(&self, new_value: bool) {
        self.with_context_data_mut(|context_data| {
            context_data.storage_readonly = new_value;
        })
    }

    pub fn get_gas_left(&self) -> u64 {
        self.with_context_data_mut(|context_data| {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };
            match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            }
        })
    }

    pub fn set_gas_left(&self, new_value: u64) {
        self.with_context_data_mut(|context_data| {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };
            set_remaining_points(instance, new_value);
        })
    }

    /// Decreases gas left by the given amount.
    /// If the amount exceeds the available gas, the remaining gas is set to 0 and
    /// an InsufficientGasLeft error is returned.
    pub fn decrease_gas_left(&self, amount: u64) -> Result<(), InsufficientGasLeft> {
        self.with_context_data_mut(|context_data| {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };

            let remaining = match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            };
            if amount > remaining {
                set_remaining_points(instance, 0);
                Err(InsufficientGasLeft)
            } else {
                set_remaining_points(instance, remaining - amount);
                Ok(())
            }
        })
    }

    pub fn memory(&self) -> Memory {
        self.with_context_data(|context_data| {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };
            let mut memories: Vec<Memory> = instance
                .exports
                .iter()
                .memories()
                .map(|pair| pair.1.clone())
                .collect();
            memories.pop().unwrap()
        })
    }

    /// Moves owned instances of storage and querier into the env.
    /// Should be followed by exactly one call to move_out when the instance is finished.
    pub fn move_in(&self, storage: S, querier: Q) {
        self.with_context_data_mut(|context_data| {
            context_data.storage = Some(storage);
            context_data.querier = Some(querier);
        });
    }

    /// Returns the original storage and querier as owned instances, and closes any remaining
    /// iterators. This is meant to be called when recycling the instance.
    pub fn move_out(&self) -> (Option<S>, Option<Q>) {
        self.with_context_data_mut(|context_data| {
            (context_data.storage.take(), context_data.querier.take())
        })
    }
}

pub struct ContextData<S: Storage, Q: Querier> {
    gas_state: GasState,
    storage: Option<S>,
    storage_readonly: bool,
    querier: Option<Q>,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
}

impl<S: Storage, Q: Querier> ContextData<S, Q> {
    pub fn new(gas_limit: u64) -> Self {
        ContextData::<S, Q> {
            gas_state: GasState::with_limit(gas_limit),
            storage: None,
            storage_readonly: true,
            querier: None,
            wasmer_instance: None,
        }
    }
}

pub fn process_gas_info<A: Api, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    info: GasInfo,
) -> VmResult<()> {
    env.decrease_gas_left(info.cost)?;
    account_for_externally_used_gas(env, info.externally_used)?;
    Ok(())
}

/// Use this function to adjust the VM's gas limit when a call into the backend
/// reported there was externally metered gas used.
/// This does not increase the VM's gas usage but ensures the overall limit is not exceeded.
fn account_for_externally_used_gas<A: Api, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    amount: u64,
) -> VmResult<()> {
    account_for_externally_used_gas_impl(env, amount)
}

fn account_for_externally_used_gas_impl<A: Api, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    used_gas: u64,
) -> VmResult<()> {
    env.with_context_data_mut(|context_data| {
        let gas_state = &mut context_data.gas_state;

        // get_gas_left implementation without a deadlock
        let gas_left = {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };
            match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            }
        };
        let wasmer_used_gas = gas_state.get_gas_used_in_wasmer(gas_left);

        gas_state.increase_externally_used_gas(used_gas);
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        let new_limit = gas_state.get_gas_left(wasmer_used_gas);

        // This tells wasmer how much more gas it can consume from this point in time.
        // set_gas_left implementation without a deadlock
        {
            let instance_ptr = context_data
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { instance_ptr.as_ref() };
            set_remaining_points(instance, new_limit);
        }

        if gas_state.externally_used_gas + wasmer_used_gas > gas_state.gas_limit {
            Err(VmError::gas_depletion())
        } else {
            Ok(())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Storage;
    use crate::conversion::ref_to_u32;
    use crate::errors::VmError;
    use crate::size::Size;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::wasm_backend::compile_and_use;
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
    };
    use wasmer::{imports, Function, Instance as WasmerInstance};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MA = MockApi;
    type MS = MockStorage;
    type MQ = MockQuerier;

    // prepared data
    const INIT_KEY: &[u8] = b"foo";
    const INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const GAS_LIMIT: u64 = 5_000_000;
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);

    fn make_instance() -> (Environment<MA, MS, MQ>, Box<WasmerInstance>) {
        let gas_limit = GAS_LIMIT;
        let env = Environment::new(MockApi::default(), gas_limit, false);

        let module = compile_and_use(&CONTRACT, TESTING_MEMORY_LIMIT).unwrap();
        let store = module.store();
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_native(&store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_native(&store, |_a: u32| {}),
                "db_scan" => Function::new_native(&store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "canonicalize_address" => Function::new_native(&store, |_a: u32, _b: u32| -> u32 { 0 }),
                "humanize_address" => Function::new_native(&store, |_a: u32, _b: u32| -> u32 { 0 }),
                "debug" => Function::new_native(&store, |_a: u32| {}),
            },
        };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(gas_limit);
        env.with_gas_state_mut(|gas_state| gas_state.set_gas_limit(gas_limit));

        (env, instance)
    }

    fn leave_default_data(env: &Environment<MA, MS, MQ>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        env.move_in(storage, querier);
    }

    #[test]
    fn move_out_works() {
        let (env, _instance) = make_instance();

        // empty data on start
        let (inits, initq) = env.move_out();
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&env);
        let (s, q) = env.move_out();
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = env.move_out();
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    fn gas_tracking_works_correctly() {
        let (env, _instance) = make_instance();

        let gas_limit = 100;
        env.set_gas_left(gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));
        assert_eq!(env.get_gas_left(), 100);

        // Consume all the Gas that we allocated
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 70).unwrap();
        assert_eq!(env.get_gas_left(), 30);
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 4).unwrap();
        assert_eq!(env.get_gas_left(), 26);
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 6).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 20).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MA, MS, MQ>(&env, 1).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let (env, _instance) = make_instance();

        let gas_limit = 100;
        env.set_gas_left(gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));
        assert_eq!(env.get_gas_left(), 100);

        // Some gas was consumed externally
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 50).unwrap();
        assert_eq!(env.get_gas_left(), 50);
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 4).unwrap();
        assert_eq!(env.get_gas_left(), 46);

        // Consume 20 gas directly in wasmer
        env.decrease_gas_left(20).unwrap();
        assert_eq!(env.get_gas_left(), 26);

        account_for_externally_used_gas::<MA, MS, MQ>(&env, 6).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        account_for_externally_used_gas::<MA, MS, MQ>(&env, 20).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MA, MS, MQ>(&env, 1).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        assert_eq!(env.is_storage_readonly(), true);
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // change
        env.set_storage_readonly(false);
        assert_eq!(env.is_storage_readonly(), false);

        // still false
        env.set_storage_readonly(false);
        assert_eq!(env.is_storage_readonly(), false);

        // change back
        env.set_storage_readonly(true);
        assert_eq!(env.is_storage_readonly(), true);
    }

    #[test]
    fn call_function_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let result = env.call_function("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result[0]).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function_fails_for_missing_instance() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // Clear context's wasmer_instance
        env.set_wasmer_instance(None);

        let res = env.call_function("allocate", &[]);
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function_fails_for_missing_function() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let res = env.call_function("doesnt_exist", &[]);
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(msg, "Could not get export: Missing export doesnt_exist");
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function0_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        env.call_function0("cosmwasm_vm_version_4", &[]).unwrap();
    }

    #[test]
    fn call_function0_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let result = env.call_function0("allocate", &[10u32.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
            } => {
                assert_eq!(function_name, "allocate");
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function1_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function1_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);

        let result = env.call_function1("deallocate", &[ptr.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
            } => {
                assert_eq!(function_name, "deallocate");
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let val = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(INIT_KEY).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        env.with_storage_from_context::<_, _>(|store| {
            store
                .set(set_key, set_value)
                .0
                .expect("error setting value");
            Ok(())
        })
        .unwrap();

        env.with_storage_from_context::<_, _>(|store| {
            assert_eq!(store.get(INIT_KEY).0.unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).0.unwrap(), Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        env.with_storage_from_context::<_, ()>(|_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let res = env
            .with_querier_from_context::<_, _>(|querier| {
                let req: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::AllBalances {
                    address: HumanAddr::from(INIT_ADDR),
                });
                let (result, _gas_info) =
                    querier.query_raw(&to_vec(&req).unwrap(), DEFAULT_QUERY_GAS_LIMIT);
                Ok(result.unwrap())
            })
            .unwrap()
            .unwrap()
            .unwrap();
        let balance: AllBalanceResponse = from_binary(&res).unwrap();

        assert_eq!(balance.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_querier_from_context_handles_panics() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        env.with_querier_from_context::<_, ()>(|_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }
}
