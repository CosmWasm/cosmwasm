//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{Function, HostEnvInitError, Instance as WasmerInstance, Memory, WasmerEnv};

use crate::backend::{GasInfo, Querier, Storage};
use crate::errors::{VmError, VmResult};
use crate::wasm_backend::{decrease_gas_left, get_gas_left, set_gas_left};

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
pub struct Environment<S: Storage, Q: Querier> {
    data: Arc<RwLock<ContextData<S, Q>>>,
    pub print_debug: bool,
}

impl<S: Storage, Q: Querier> Clone for Environment<S, Q> {
    fn clone(&self) -> Self {
        Environment {
            data: self.data.clone(),
            print_debug: self.print_debug,
        }
    }
}

impl<S: Storage, Q: Querier> WasmerEnv for Environment<S, Q> {
    fn init_with_instance(&mut self, _instance: &WasmerInstance) -> Result<(), HostEnvInitError> {
        Ok(())
    }
}

impl<S: Storage, Q: Querier> Environment<S, Q> {
    pub fn new(gas_limit: u64, print_debug: bool) -> Self {
        Environment {
            data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
            print_debug,
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

    pub fn with_func_from_context<C, T>(&self, name: &str, callback: C) -> VmResult<T>
    where
        C: FnOnce(&Function) -> VmResult<T>,
    {
        self.with_context_data_mut(|context_data| match context_data.wasmer_instance {
            Some(instance_ptr) => {
                let func = unsafe { instance_ptr.as_ref() }
                    .exports
                    .get_function(name)?;
                callback(func)
            }
            None => Err(VmError::uninitialized_context_data("wasmer_instance")),
        })
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

    pub fn memory(&self) -> Memory {
        self.with_context_data(|context| {
            let ptr = context
                .wasmer_instance
                .expect("Wasmer instance is not set. This is a bug.");
            let instance = unsafe { ptr.as_ref() };
            let mut memories: Vec<Memory> = instance
                .exports
                .iter()
                .memories()
                .map(|pair| pair.1.clone())
                .collect();
            memories.pop().unwrap()
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

/// Returns the original storage and querier as owned instances, and closes any remaining
/// iterators. This is meant to be called when recycling the instance.
pub(crate) fn move_out_of_environment<S: Storage, Q: Querier>(
    env: &Environment<S, Q>,
) -> (Option<S>, Option<Q>) {
    env.with_context_data_mut(|context_data| {
        (context_data.storage.take(), context_data.querier.take())
    })
}

/// Moves owned instances of storage and querier into the env.
/// Should be followed by exactly one call to move_out_of_environment when the instance is finished.
pub(crate) fn move_into_environment<S: Storage, Q: Querier>(
    env: &Environment<S, Q>,
    storage: S,
    querier: Q,
) {
    env.with_context_data_mut(|context_data| {
        context_data.storage = Some(storage);
        context_data.querier = Some(querier);
    });
}

pub fn process_gas_info<S: Storage, Q: Querier>(
    env: &Environment<S, Q>,
    info: GasInfo,
) -> VmResult<()> {
    decrease_gas_left(env, info.cost)?;
    account_for_externally_used_gas(env, info.externally_used)?;
    Ok(())
}

/// Use this function to adjust the VM's gas limit when a call into the backend
/// reported there was externally metered gas used.
/// This does not increase the VM's gas usage but ensures the overall limit is not exceeded.
fn account_for_externally_used_gas<S: Storage, Q: Querier>(
    env: &Environment<S, Q>,
    amount: u64,
) -> VmResult<()> {
    account_for_externally_used_gas_impl(env, amount)
}

fn account_for_externally_used_gas_impl<S: Storage, Q: Querier>(
    env: &Environment<S, Q>,
    used_gas: u64,
) -> VmResult<()> {
    env.with_context_data_mut(|context_data| {
        let gas_state = &mut context_data.gas_state;

        let wasmer_used_gas = gas_state.get_gas_used_in_wasmer(get_gas_left(env));

        gas_state.increase_externally_used_gas(used_gas);
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        let new_limit = gas_state.get_gas_left(wasmer_used_gas);
        // This tells wasmer how much more gas it can consume from this point in time.
        set_gas_left(env, new_limit);

        if gas_state.externally_used_gas + wasmer_used_gas > gas_state.gas_limit {
            Err(VmError::GasDepletion)
        } else {
            Ok(())
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::Storage;
    use crate::conversion::ref_to_u32;
    use crate::errors::VmError;
    use crate::size::Size;
    use crate::testing::{MockQuerier, MockStorage};
    #[cfg(feature = "metering")]
    use crate::wasm_backend::decrease_gas_left;
    use crate::wasm_backend::{compile, set_gas_left};
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
    };
    use wasmer::{imports, Function, FunctionType, Instance as WasmerInstance, Type, Val};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
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

    fn make_instance() -> (Environment<MS, MQ>, Box<WasmerInstance>) {
        let env = Environment::new(GAS_LIMIT, false);

        let module = compile(&CONTRACT, Some(TESTING_MEMORY_LIMIT)).unwrap();
        let store = module.store();
        let i32_to_void = FunctionType::new(vec![Type::I32], vec![]);
        let i32_to_i32 = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let i32i32_to_void = FunctionType::new(vec![Type::I32, Type::I32], vec![]);
        let i32i32_to_i32 = FunctionType::new(vec![Type::I32, Type::I32], vec![Type::I32]);
        let i32i32i32_to_i32 =
            FunctionType::new(vec![Type::I32, Type::I32, Type::I32], vec![Type::I32]);
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new(store, &i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "db_write" => Function::new(store, &i32i32_to_void, |_args: &[Val]| { Ok(vec![]) }),
                "db_remove" => Function::new(store, &i32_to_void, |_args: &[Val]| { Ok(vec![]) }),
                "db_scan" => Function::new(store, &i32i32i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "db_next" => Function::new(store, &i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "query_chain" => Function::new(store, &i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "canonicalize_address" => Function::new(store, &i32i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "humanize_address" => Function::new(store, &i32i32_to_i32, |_args: &[Val]| { Ok(vec![Val::I32(0)]) }),
                "debug" => Function::new(store, &i32_to_void, |_args: &[Val]| { Ok(vec![]) }),
            },
        };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));

        (env, instance)
    }

    fn leave_default_data(env: &Environment<MS, MQ>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_environment(env, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        let (env, _instance) = make_instance();

        // empty data on start
        let (inits, initq) = move_out_of_environment::<MS, MQ>(&env);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&env);
        let (s, q) = move_out_of_environment::<MS, MQ>(&env);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = move_out_of_environment::<MS, MQ>(&env);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    fn gas_tracking_works_correctly() {
        let (env, _instance) = make_instance();

        let gas_limit = 100;
        set_gas_left(&env, gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));

        // Consume all the Gas that we allocated
        account_for_externally_used_gas::<MS, MQ>(&env, 70).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&env, 4).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&env, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&env, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&env, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "metering")]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let (env, _instance) = make_instance();

        let gas_limit = 100;
        set_gas_left(&env, gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));

        // Some gas was consumed externally
        account_for_externally_used_gas::<MS, MQ>(&env, 50).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&env, 4).unwrap();

        // Consume 20 gas directly in wasmer
        decrease_gas_left(&env, 20).unwrap();

        account_for_externally_used_gas::<MS, MQ>(&env, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&env, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&env, 1).unwrap_err() {
            VmError::GasDepletion => {}
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
    fn with_func_from_context_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let ptr = env
            .with_func_from_context::<_, _>("allocate", |alloc_func| {
                let result = alloc_func.call(&[10u32.into()])?;
                let ptr = ref_to_u32(&result[0])?;
                Ok(ptr)
            })
            .unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn with_func_from_context_fails_for_missing_instance() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // Clear context's wasmer_instance
        env.set_wasmer_instance(None);

        let res = env.with_func_from_context::<_, ()>("allocate", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_func_from_context_fails_for_missing_function() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let res = env.with_func_from_context::<_, ()>("doesnt_exist", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(msg, "Could not get export: Missing export doesnt_exist");
            }
            err => panic!("Unexpected error: {:?}", err),
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
