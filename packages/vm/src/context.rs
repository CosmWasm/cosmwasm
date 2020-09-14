//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
#[cfg(feature = "iterator")]
use std::collections::HashMap;
#[cfg(feature = "iterator")]
use std::convert::TryInto;
#[cfg(not(feature = "iterator"))]
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{Function, Instance as WasmerInstance};

use crate::backends::decrease_gas_left;
use crate::errors::{VmError, VmResult};
use crate::ffi::GasInfo;
#[cfg(feature = "iterator")]
use crate::traits::StorageIterator;
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
    /// Get a mutable reference to the context's data. Ownership remains in the Context.
    // NOTE: This is actually not really implemented safely at the moment. I did this as a
    // nicer and less-terrible version of the previous solution to the following issue:
    //
    //                                                   +--->> Go pointer
    //                                                   |
    // Env ->> ContextData +-> iterators: Box<dyn Iterator + 'a> --+
    //                     |                                       |
    //                     +-> storage: impl Storage <<------------+
    //                     |
    //                     +-> querier: impl Querier
    //
    // ->  : Ownership
    // ->> : Mutable borrow
    //
    // As you can see, there's a cyclical reference here... changing this function to return the same lifetime as it
    // returns (and adjusting a few other functions to only have one lifetime instead of two) triggers an error
    // elsewhere where we try to add iterators to the context. That's not legal according to Rust's rules, and it
    // complains that we're trying to borrow ctx mutably twice. This needs a better solution because this function
    // probably triggers unsoundness.
    // pub fn get_context_data_mut(&mut self) -> &mut ContextData<S, Q> {
    //     let mut guard = self.context_data.as_ref().write().unwrap();
    //     guard.borrow_mut()
    // }

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
    #[cfg(feature = "iterator")]
    iterators: HashMap<u32, Box<dyn StorageIterator + 'a>>,
    #[cfg(not(feature = "iterator"))]
    iterators: PhantomData<()>,
}

impl<S: Storage, Q: Querier> ContextData<S, Q> {
    pub fn new(gas_limit: u64) -> Self {
        ContextData::<S, Q> {
            gas_state: GasState::with_limit(gas_limit),
            storage: None,
            storage_readonly: true,
            querier: None,
            wasmer_instance: None,
            #[cfg(feature = "iterator")]
            iterators: HashMap::new(),
            #[cfg(not(feature = "iterator"))]
            iterators: PhantomData::default(),
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

#[cfg(feature = "iterator")]
fn destroy_iterators<S: Storage, Q: Querier>(context: &mut ContextData<S, Q>) {
    context.iterators.clear();
}

#[cfg(not(feature = "iterator"))]
fn destroy_iterators<S: Storage, Q: Querier>(_context: &mut ContextData<S, Q>) {}

/// Returns the original storage and querier as owned instances, and closes any remaining
/// iterators. This is meant to be called when recycling the instance.
pub(crate) fn move_out_of_context<S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
) -> (Option<S>, Option<Q>) {
    env.with_context_data_mut(|context_data| {
        // Destroy all existing iterators which are (in contrast to the storage)
        // not reused between different instances.
        // This is also important because the iterators are pointers to Go memory which should not be stored long term
        // Paragraphs 5-7: https://golang.org/cmd/cgo/#hdr-Passing_pointers
        destroy_iterators(context_data);
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

/// Add the iterator to the context's data. A new ID is assigned and returned.
/// IDs are guaranteed to be in the range [0, 2**31-1], i.e. fit in the non-negative part if type i32.
#[cfg(feature = "iterator")]
#[must_use = "without the returned iterator ID, the iterator cannot be accessed"]
pub fn add_iterator<'a, S: Storage, Q: Querier>(
    env: &mut Env<S, Q>,
    iter: Box<dyn StorageIterator + 'a>,
) -> u32 {
    env.with_context_data_mut(|context_data| {
        let last_id: u32 = context_data
            .iterators
            .len()
            .try_into()
            .expect("Found more iterator IDs than supported");
        let new_id = last_id + 1;
        if new_id > (i32::MAX as u32) {
            panic!("Iterator ID exceeded i32::MAX. This must not happen.");
        }
        context_data.iterators.insert(new_id, iter);
        new_id
    })
}

// TODO: move into Env
pub(crate) fn with_func_from_context<S, Q, Callback, CallbackData>(
    mut env: Env<S, Q>,
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
pub(crate) fn with_storage_from_context<S, Q, F, T>(mut env: Env<S, Q>, func: F) -> VmResult<T>
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
pub(crate) fn with_querier_from_context<S, Q, F, T>(mut env: Env<S, Q>, func: F) -> VmResult<T>
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

// TODO: move into Env
#[cfg(feature = "iterator")]
pub(crate) fn with_iterator_from_context<S, Q, F, T>(
    env: Env<S, Q>,
    iterator_id: u32,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&mut (dyn StorageIterator)) -> VmResult<T>,
{
    env.with_context_data_mut(
        |context_data| match context_data.iterators.get_mut(&iterator_id) {
            Some(iterator) => func(iterator),
            None => Err(VmError::iterator_does_not_exist(iterator_id)),
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backends::{compile, decrease_gas_left, set_gas_left};
    use crate::errors::VmError;
    #[cfg(feature = "iterator")]
    use crate::testing::MockIterator;
    use crate::testing::{MockQuerier, MockStorage};
    use crate::traits::Storage;
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
    };
    use wasmer::imports;

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

    fn make_instance() -> Box<WasmerInstance> {
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
        set_wasmer_instance::<MS, MQ>(&mut instance.context_mut(), Some(instance_ptr));

        instance
    }

    fn leave_default_data(ctx: &mut Ctx) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(ctx, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        // this creates an instance
        let mut instance = make_instance();
        let ctx = instance.context_mut();

        // empty data on start
        let (inits, initq) = move_out_of_context::<MS, MQ>(&mut ctx);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&mut ctx);
        let (s, q) = move_out_of_context::<MS, MQ>(&mut ctx);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = move_out_of_context::<MS, MQ>(&mut ctx);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_left(&mut instance.context_mut(), gas_limit);
        get_gas_state_mut::<MS, MQ>(&mut instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Consume all the Gas that we allocated
        account_for_externally_used_gas::<MS, MQ>(&mut context, 70).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 4).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&mut context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_left(&mut instance.context_mut(), gas_limit);
        get_gas_state_mut::<MS, MQ>(&mut instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Some gas was consumed externally
        account_for_externally_used_gas::<MS, MQ>(&mut context, 50).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 4).unwrap();

        // Consume 20 gas directly in wasmer
        decrease_gas_left(&mut instance.context_mut(), 20).unwrap();

        let context = instance.context_mut();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(&mut context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(&mut context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        assert_eq!(is_storage_readonly::<MS, MQ>(&ctx), true);
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        // change
        set_storage_readonly::<MS, MQ>(&mut ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(&ctx), false);

        // still false
        set_storage_readonly::<MS, MQ>(&mut ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(&ctx), false);

        // change back
        set_storage_readonly::<MS, MQ>(&mut ctx, true);
        assert_eq!(is_storage_readonly::<MS, MQ>(&ctx), true);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn add_iterator_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        assert_eq!(get_context_data_mut::<MS, MQ>(ctx).iterators.len(), 0);
        let id1 = add_iterator::<MS, MQ>(ctx, Box::new(MockIterator::empty()));
        let id2 = add_iterator::<MS, MQ>(ctx, Box::new(MockIterator::empty()));
        let id3 = add_iterator::<MS, MQ>(ctx, Box::new(MockIterator::empty()));
        assert_eq!(get_context_data_mut::<MS, MQ>(ctx).iterators.len(), 3);
        assert!(get_context_data_mut::<MS, MQ>(ctx)
            .iterators
            .contains_key(&id1));
        assert!(get_context_data_mut::<MS, MQ>(ctx)
            .iterators
            .contains_key(&id2));
        assert!(get_context_data_mut::<MS, MQ>(ctx)
            .iterators
            .contains_key(&id3));
    }

    #[test]
    fn with_func_from_context_works() {
        let mut instance = make_instance();
        leave_default_data(&mut instance.context_mut());

        let ctx = instance.context_mut();
        let ptr =
            with_func_from_context::<MS, MQ, u32, u32, _, _>(&mut ctx, "allocate", |alloc_func| {
                let ptr = alloc_func.call(10)?;
                Ok(ptr)
            })
            .unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn with_func_from_context_fails_for_missing_instance() {
        let mut instance = make_instance();
        leave_default_data(&mut instance.context_mut());

        // Clear context's wasmer_instance
        set_wasmer_instance::<MS, MQ>(&mut instance.context_mut(), None);

        let ctx = instance.context_mut();
        let res =
            with_func_from_context::<MS, MQ, u32, u32, _, ()>(&mut ctx, "allocate", |_func| {
                panic!("unexpected callback call");
            });
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            e => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn with_func_from_context_fails_for_missing_function() {
        let mut instance = make_instance();
        leave_default_data(&mut instance.context_mut());

        let ctx = instance.context_mut();
        let res =
            with_func_from_context::<MS, MQ, u32, u32, _, ()>(&mut ctx, "doesnt_exist", |_func| {
                panic!("unexpected callback call");
            });
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasmer resolve error: ExportNotFound { name: \"doesnt_exist\" }"
                );
            }
            e => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        let val = with_storage_from_context::<MS, MQ, _, _>(&mut ctx, |store| {
            Ok(store.get(INIT_KEY).0.expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        with_storage_from_context::<MS, MQ, _, _>(&mut &mut ctx, |store| {
            store
                .set(set_key, set_value)
                .0
                .expect("error setting value");
            Ok(())
        })
        .unwrap();

        with_storage_from_context::<MS, MQ, _, _>(&mut ctx, |store| {
            assert_eq!(store.get(INIT_KEY).0.unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).0.unwrap(), Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        with_storage_from_context::<MS, MQ, _, ()>(&mut ctx, |_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        let res = with_querier_from_context::<MS, MQ, _, _>(&mut ctx, |querier| {
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
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(&mut ctx);

        with_querier_from_context::<MS, MQ, _, ()>(&mut ctx, |_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn with_iterator_from_context_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = add_iterator::<MS, MQ>(ctx, Box::new(MockIterator::empty()));
        with_iterator_from_context::<MS, MQ, _, ()>(ctx, id, |iter| {
            assert!(iter.next().0.unwrap().is_none());
            Ok(())
        })
        .expect("must not error");
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn with_iterator_from_context_errors_for_non_existent_iterator_id() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let missing_id = 42u32;
        let result = with_iterator_from_context::<MS, MQ, _, ()>(ctx, missing_id, |_iter| {
            panic!("this should not be called");
        });
        match result.unwrap_err() {
            VmError::IteratorDoesNotExist { id, .. } => assert_eq!(id, missing_id),
            e => panic!("Unexpected error: {}", e),
        }
    }
}
