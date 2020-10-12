//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
#[cfg(feature = "iterator")]
use std::collections::HashMap;
#[cfg(feature = "iterator")]
use std::convert::TryInto;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{Function, Instance as WasmerInstance};

use crate::backends::decrease_gas_left;
use crate::errors::{VmError, VmResult};
use crate::ffi::GasInfo;
use crate::traits::{Querier, Storage};

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

// #[derive(Clone)]
pub struct Env<S: Storage, Q: Querier> {
    pub memory: wasmer::Memory,
    pub context_data: Arc<RwLock<ContextData<S, Q>>>,
}

impl<S: Storage, Q: Querier> Clone for Env<S, Q> {
    fn clone(&self) -> Self {
        Env {
            memory: self.memory.clone(),
            context_data: self.context_data.clone(),
        }
    }
}

impl<S: Storage, Q: Querier> Env<S, Q> {
    pub fn with_context_data_mut<Callback, CallbackReturn>(
        &mut self,
        callback: Callback,
    ) -> CallbackReturn
    where
        Callback: FnOnce(&mut ContextData<S, Q>) -> CallbackReturn,
    {
        let mut guard = self.context_data.as_ref().write().unwrap();
        let context_data = guard.borrow_mut();
        callback(context_data)
    }

    pub fn with_context_data<Callback, CallbackReturn>(&self, callback: Callback) -> CallbackReturn
    where
        Callback: FnOnce(&ContextData<S, Q>) -> CallbackReturn,
    {
        let guard = self.context_data.as_ref().read().unwrap();
        let context_data = guard.borrow();
        callback(context_data)
    }

    pub fn with_gas_state_mut<Callback, CallbackReturn>(
        &mut self,
        callback: Callback,
    ) -> CallbackReturn
    where
        Callback: FnOnce(&mut GasState) -> CallbackReturn,
    {
        self.with_context_data_mut(|context_data| callback(&mut context_data.gas_state))
    }

    pub fn with_gas_state<Callback, CallbackReturn>(&self, callback: Callback) -> CallbackReturn
    where
        Callback: FnOnce(&GasState) -> CallbackReturn,
    {
        self.with_context_data(|context_data| callback(&context_data.gas_state))
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

/// Creates a back reference from a contact to its partent instance
pub fn set_wasmer_instance<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
    wasmer_instance: Option<NonNull<WasmerInstance>>,
) {
    env.with_context_data_mut(|context_data| {
        context_data.wasmer_instance = wasmer_instance;
    });
}

/// Returns the original storage and querier as owned instances, and closes any remaining
/// iterators. This is meant to be called when recycling the instance.
pub(crate) fn move_out_of_context<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
) -> (Option<S>, Option<Q>) {
    env.with_context_data_mut(|context_data| {
        (context_data.storage.take(), context_data.querier.take())
    })
}

/// Moves owned instances of storage and querier into the env.
/// Should be followed by exactly one call to move_out_of_context when the instance is finished.
pub(crate) fn move_into_context<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
    storage: S,
    querier: Q,
) {
    env.with_context_data_mut(|context_data| {
        context_data.storage = Some(storage);
        context_data.querier = Some(querier);
    });
}

pub fn process_gas_info<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
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
    env: &mut Env<S, Q>,
    amount: u64,
) -> VmResult<()> {
    account_for_externally_used_gas_impl(env, amount)
}

#[cfg(feature = "default-singlepass")]
fn account_for_externally_used_gas_impl<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
    used_gas: u64,
) -> VmResult<()> {
    use crate::backends::{get_gas_left, set_gas_left};

    // WFT?!
    let mut env1 = env.clone();
    let env2 = env.clone();
    let mut env3 = env.clone();

    env1.with_context_data_mut(|context_data| {
        let gas_state = &mut context_data.gas_state;

        let wasmer_used_gas = gas_state.get_gas_used_in_wasmer(get_gas_left(&env2));

        gas_state.increase_externally_used_gas(used_gas);
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        let new_limit = gas_state.get_gas_left(wasmer_used_gas);
        // This tells wasmer how much more gas it can consume from this point in time.
        set_gas_left(&mut env3, new_limit);

        if gas_state.externally_used_gas + wasmer_used_gas > gas_state.gas_limit {
            Err(VmError::GasDepletion)
        } else {
            Ok(())
        }
    })
}

#[cfg(feature = "default-cranelift")]
fn account_for_externally_used_gas_impl<S: Storage, Q: Querier>(
    _ctx: &mut Ctx,
    _used_gas: u64,
) -> VmResult<()> {
    Ok(())
}

/// Returns true iff the storage is set to readonly mode
pub fn is_storage_readonly<S: Storage, Q: Querier>(env: &Env<S, Q>) -> bool {
    env.with_context_data(|context_data| context_data.storage_readonly)
}

pub fn set_storage_readonly<S: Storage, Q: Querier>(env: &mut Env<S, Q>, new_value: bool) {
    env.with_context_data_mut(|context_data| {
        context_data.storage_readonly = new_value;
    })
}

// TODO: move into Env
pub(crate) fn with_func_from_context<S, Q, Callback, CallbackData>(
    env: &mut Env<S, Q>,
    name: &str,
    callback: Callback,
) -> VmResult<CallbackData>
where
    S: Storage,
    Q: Querier,
    Callback: FnOnce(&Function) -> VmResult<CallbackData>,
{
    env.with_context_data_mut(|context_data| match context_data.wasmer_instance {
        Some(instance_ptr) => {
            let func = unsafe { instance_ptr.as_ref() }
                .exports
                .get_function(name)?;
            callback(func)
        }
        None => Err(VmError::uninitialized_context_data("wasmer_instance")),
    })
}

// TODO: move into Env
pub(crate) fn with_storage_from_context<S, Q, F, T>(env: &mut Env<S, Q>, func: F) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&mut S) -> VmResult<T>,
{
    env.with_context_data_mut(|context_data| match context_data.storage.as_mut() {
        Some(data) => func(data),
        None => Err(VmError::uninitialized_context_data("storage")),
    })
}

// TODO: move into Env
pub(crate) fn with_querier_from_context<S, Q, F, T>(env: &mut Env<S, Q>, func: F) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&mut Q) -> VmResult<T>,
{
    env.with_context_data_mut(|context_data| match context_data.querier.as_mut() {
        Some(querier) => func(querier),
        None => Err(VmError::uninitialized_context_data("querier")),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backends::{compile, decrease_gas_left, set_gas_left};
    use crate::errors::VmError;
    use crate::testing::{MockQuerier, MockStorage};
    use crate::traits::Storage;
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
    };
    use std::sync::{Arc, RwLock};
    use wasmer::{imports, Store};
    use wasmer_engine_jit::JIT;

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

    fn make_instance() -> (Env<MS, MQ>, Box<WasmerInstance>) {
        let engine = JIT::headless().engine();
        let store = Store::new(&engine);

        let mut env = Env {
            memory: wasmer::Memory::new(&store, wasmer::MemoryType::new(0, Some(5000), false))
                .expect("could not create memory"),
            context_data: Arc::new(RwLock::new(ContextData::new(GAS_LIMIT))),
        };

        let module = compile(&CONTRACT).unwrap();
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                // "db_read" => Func::new(|_ctx: &mut Ctx, _a: u32| -> u32 { 0 }),
                // "db_write" => Func::new(|_ctx: &mut Ctx, _a: u32, _b: u32| {}),
                // "db_remove" => Func::new(|_ctx: &mut Ctx, _a: u32| {}),
                // "db_scan" => Func::new(|_ctx: &mut Ctx, _a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                // "db_next" => Func::new(|_ctx: &mut Ctx, _a: u32| -> u32 { 0 }),
                // "query_chain" => Func::new(|_ctx: &mut Ctx, _a: u32| -> u32 { 0 }),
                // "canonicalize_address" => Func::new(|_ctx: &mut Ctx, _a: u32, _b: u32| -> u32 { 0 }),
                // "humanize_address" => Func::new(|_ctx: &mut Ctx, _a: u32, _b: u32| -> u32 { 0 }),
                // "debug" => Func::new(|_ctx: &mut Ctx, _a: u32| {}),
            },
        };
        let mut instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        set_wasmer_instance::<MS, MQ>(&mut env, Some(instance_ptr));

        (env, instance)
    }

    fn leave_default_data(env: &mut Env<MS, MQ>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(env, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        let (mut env, mut instance) = make_instance();

        // empty data on start
        let (inits, initq) = move_out_of_context::<MS, MQ>(&mut env);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&mut env);
        let (s, q) = move_out_of_context::<MS, MQ>(&mut env);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = move_out_of_context::<MS, MQ>(&mut env);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly() {
        let (mut env, mut instance) = make_instance();

        let gas_limit = 100;
        set_gas_left(&mut env, gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));

        // Consume all the Gas that we allocated
        account_for_externally_used_gas::<MS, MQ>(&mut env, 70).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut env, 4).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut env, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut env, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&mut env, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let (mut env, mut instance) = make_instance();

        let gas_limit = 100;
        set_gas_left(&mut env, gas_limit);
        env.with_gas_state_mut(|state| state.set_gas_limit(gas_limit));

        // Some gas was consumed externally
        account_for_externally_used_gas::<MS, MQ>(&mut env, 50).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut env, 4).unwrap();

        // Consume 20 gas directly in wasmer
        decrease_gas_left(&mut env, 20).unwrap();

        account_for_externally_used_gas::<MS, MQ>(&mut env, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut env, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&mut env, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        assert_eq!(is_storage_readonly::<MS, MQ>(&env), true);
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        // change
        set_storage_readonly::<MS, MQ>(&mut env, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(&env), false);

        // still false
        set_storage_readonly::<MS, MQ>(&mut env, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(&env), false);

        // change back
        set_storage_readonly::<MS, MQ>(&mut env, true);
        assert_eq!(is_storage_readonly::<MS, MQ>(&env), true);
    }

    #[test]
    fn with_func_from_context_works() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        let ptr = with_func_from_context::<MS, MQ, _, _>(&mut env, "allocate", |alloc_func| {
            let result = alloc_func.call(&[10u32.into()])?;
            let ptr = result[0].unwrap_i32() as u32;
            Ok(ptr)
        })
        .unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn with_func_from_context_fails_for_missing_instance() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        // Clear context's wasmer_instance
        set_wasmer_instance::<MS, MQ>(&mut env, None);

        let res = with_func_from_context::<MS, MQ, _, ()>(&mut env, "allocate", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_func_from_context_fails_for_missing_function() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        let res = with_func_from_context::<MS, MQ, _, ()>(&mut env, "doesnt_exist", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasmer resolve error: ExportNotFound { name: \"doesnt_exist\" }"
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        let val = with_storage_from_context::<MS, MQ, _, _>(&mut env, |store| {
            Ok(store.get(INIT_KEY).0.expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        with_storage_from_context::<MS, MQ, _, _>(&mut env, |store| {
            store
                .set(set_key, set_value)
                .0
                .expect("error setting value");
            Ok(())
        })
        .unwrap();

        with_storage_from_context::<MS, MQ, _, _>(&mut env, |store| {
            assert_eq!(store.get(INIT_KEY).0.unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).0.unwrap(), Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        with_storage_from_context::<MS, MQ, _, ()>(&mut env, |_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        let res = with_querier_from_context::<MS, MQ, _, _>(&mut env, |querier| {
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
        let (mut env, mut instance) = make_instance();
        leave_default_data(&mut env);

        with_querier_from_context::<MS, MQ, _, ()>(&mut env, |_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }
}
