//! Internal details to be used by instance.rs only
#[cfg(feature = "iterator")]
use std::collections::HashMap;
#[cfg(feature = "iterator")]
use std::convert::TryInto;
use std::ffi::c_void;
#[cfg(not(feature = "iterator"))]
use std::marker::PhantomData;
use std::ptr::NonNull;

use wasmer_runtime_core::{
    typed_func::{Func, Wasm, WasmTypeList},
    vm::Ctx,
    Instance as WasmerInstance,
};

use crate::errors::{VmError, VmResult};
#[cfg(feature = "iterator")]
use crate::traits::StorageIterator;
use crate::traits::{Querier, Storage};

/** context data **/

#[derive(Clone, PartialEq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation.
    gas_limit: u64,
    /// Tracking the gas used in the cosmos SDK, in cosmwasm units.
    #[allow(unused)]
    externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }

    #[allow(unused)]
    fn use_gas(&mut self, amount: u64) {
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
    fn get_gas_used_in_wasmer(&self, wasmer_gas_left: u64) -> u64 {
        self.gas_limit
            .saturating_sub(self.externally_used_gas)
            .saturating_sub(wasmer_gas_left)
    }
}

struct ContextData<'a, S: Storage, Q: Querier> {
    gas_state: GasState,
    storage: Option<S>,
    storage_readonly: bool,
    querier: Option<Q>,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
    #[cfg(feature = "iterator")]
    iterators: HashMap<u32, Box<dyn StorageIterator + 'a>>,
    #[cfg(not(feature = "iterator"))]
    iterators: PhantomData<&'a mut ()>,
}

pub fn setup_context<S: Storage, Q: Querier>(gas_limit: u64) -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_context_data::<S, Q>(gas_limit),
        destroy_unmanaged_context_data::<S, Q>,
    )
}

fn create_unmanaged_context_data<S: Storage, Q: Querier>(gas_limit: u64) -> *mut c_void {
    let data = ContextData::<S, Q> {
        gas_state: GasState::with_limit(gas_limit),
        storage: None,
        storage_readonly: true,
        querier: None,
        wasmer_instance: None,
        #[cfg(feature = "iterator")]
        iterators: HashMap::new(),
        #[cfg(not(feature = "iterator"))]
        iterators: PhantomData::default(),
    };
    let heap_data = Box::new(data); // move from stack to heap
    Box::into_raw(heap_data) as *mut c_void // give up ownership
}

fn destroy_unmanaged_context_data<S: Storage, Q: Querier>(ptr: *mut c_void) {
    if !ptr.is_null() {
        // obtain ownership and drop instance of ContextData when box gets out of scope
        let mut dying = unsafe { Box::from_raw(ptr as *mut ContextData<S, Q>) };
        // Ensure all iterators are dropped before the storage
        destroy_iterators(&mut dying);
    }
}

/// Get a mutable reference to the context's data. Ownership remains in the Context.
// NOTE: This is actually not really implemented safely at the moment. I did this as a
// nicer and less-terrible version of the previous solution to the following issue:
//
//                                                   +--->> Go pointer
//                                                   |
// Ctx ->> ContextData +-> iterators: Box<dyn Iterator + 'a> --+
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
fn get_context_data_mut<'a, 'b, S: Storage, Q: Querier>(
    ctx: &'a mut Ctx,
) -> &'b mut ContextData<'b, S, Q> {
    let owned = unsafe {
        Box::from_raw(ctx.data as *mut ContextData<S, Q>) // obtain ownership
    };
    Box::leak(owned) // give up ownership
}

fn get_context_data<'a, 'b, S: Storage, Q: Querier>(ctx: &'a Ctx) -> &'b ContextData<'b, S, Q> {
    let owned = unsafe {
        Box::from_raw(ctx.data as *mut ContextData<S, Q>) // obtain ownership
    };
    Box::leak(owned) // give up ownership
}

/// Creates a back reference from a contact to its partent instance
pub fn set_wasmer_instance<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    wasmer_instance: Option<NonNull<WasmerInstance>>,
) {
    let context_data = ctx.data as *mut ContextData<S, Q>;
    unsafe {
        (*context_data).wasmer_instance = wasmer_instance;
    }
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
    source: &mut Ctx,
) -> (Option<S>, Option<Q>) {
    let mut b = get_context_data_mut::<S, Q>(source);
    // Destroy all existing iterators which are (in contrast to the storage)
    // not reused between different instances.
    // This is also important because the iterators are pointers to Go memory which should not be stored long term
    // Paragraphs 5-7: https://golang.org/cmd/cgo/#hdr-Passing_pointers
    destroy_iterators(&mut b);
    (b.storage.take(), b.querier.take())
}

/// Moves owned instances of storage and querier into the context.
/// Should be followed by exactly one call to move_out_of_context when the instance is finished.
pub(crate) fn move_into_context<S: Storage, Q: Querier>(target: &mut Ctx, storage: S, querier: Q) {
    let b = get_context_data_mut::<S, Q>(target);
    b.storage = Some(storage);
    b.querier = Some(querier);
}

pub fn get_gas_state<'a, 'b, S: Storage, Q: Querier + 'b>(ctx: &'a mut Ctx) -> &'b mut GasState {
    &mut get_context_data_mut::<S, Q>(ctx).gas_state
}

#[cfg(feature = "default-singlepass")]
pub fn try_consume_gas<S: Storage, Q: Querier>(ctx: &mut Ctx, used_gas: u64) -> VmResult<()> {
    use crate::backends::{get_gas_left, set_gas_limit};

    let ctx_data = get_context_data_mut::<S, Q>(ctx);
    if let Some(mut instance_ptr) = ctx_data.wasmer_instance {
        let instance = unsafe { instance_ptr.as_mut() };
        let gas_state = &mut ctx_data.gas_state;

        let wasmer_used_gas = gas_state.get_gas_used_in_wasmer(get_gas_left(instance));

        gas_state.use_gas(used_gas);
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        let new_limit = gas_state.get_gas_left(wasmer_used_gas);
        // This tells wasmer how much more gas it can consume from this point in time.
        set_gas_limit(instance, new_limit);

        if gas_state.externally_used_gas + wasmer_used_gas > gas_state.gas_limit {
            Err(VmError::GasDepletion)
        } else {
            Ok(())
        }
    } else {
        Err(VmError::uninitialized_context_data("wasmer_instance"))
    }
}

#[cfg(feature = "default-cranelift")]
pub fn try_consume_gas<S: Storage, Q: Querier>(_ctx: &mut Ctx, _used_gas: u64) -> VmResult<()> {
    Ok(())
}

/// Returns true iff the storage is set to readonly mode
pub fn is_storage_readonly<S: Storage, Q: Querier>(ctx: &Ctx) -> bool {
    let context_data = get_context_data::<S, Q>(ctx);
    context_data.storage_readonly
}

pub fn set_storage_readonly<S: Storage, Q: Querier>(ctx: &mut Ctx, new_value: bool) {
    let mut context_data = get_context_data_mut::<S, Q>(ctx);
    context_data.storage_readonly = new_value;
}

/// Add the iterator to the context's data. A new ID is assigned and returned.
/// IDs are guaranteed to be in the range [0, 2**31-1], i.e. fit in the non-negative part if type i32.
#[cfg(feature = "iterator")]
#[must_use = "without the returned iterator ID, the iterator cannot be accessed"]
pub fn add_iterator<'a, S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    iter: Box<dyn StorageIterator + 'a>,
) -> u32 {
    let b = get_context_data_mut::<S, Q>(ctx);
    let last_id: u32 = b
        .iterators
        .len()
        .try_into()
        .expect("Found more iterator IDs than supported");
    let new_id = last_id + 1;
    static INT32_MAX_VALUE: u32 = 2_147_483_647;
    if new_id > INT32_MAX_VALUE {
        panic!("Iterator ID exceeded INT32_MAX_VALUE. This must not happen.");
    }
    b.iterators.insert(new_id, iter);
    new_id
}

pub(crate) fn with_func_from_context<S, Q, Args, Rets, Callback, CallbackData>(
    ctx: &mut Ctx,
    name: &str,
    callback: Callback,
) -> VmResult<CallbackData>
where
    S: Storage,
    Q: Querier,
    Args: WasmTypeList,
    Rets: WasmTypeList,
    Callback: FnOnce(Func<Args, Rets, Wasm>) -> VmResult<CallbackData>,
{
    let ctx_data = get_context_data::<S, Q>(ctx);
    match ctx_data.wasmer_instance {
        Some(instance_ptr) => {
            let func = unsafe { instance_ptr.as_ref() }.exports.get(name)?;
            callback(func)
        }
        None => Err(VmError::uninitialized_context_data("wasmer_instance")),
    }
}

pub(crate) fn with_storage_from_context<'a, 'b, S, Q: 'b, F, T>(
    ctx: &'a mut Ctx,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&'b mut S) -> VmResult<T>,
{
    let b = get_context_data_mut::<S, Q>(ctx);
    match b.storage.as_mut() {
        Some(data) => func(data),
        None => Err(VmError::uninitialized_context_data("storage")),
    }
}

pub(crate) fn with_querier_from_context<'a, 'b, S, Q: 'b, F, T>(
    ctx: &'a mut Ctx,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&'b mut Q) -> VmResult<T>,
{
    let b = get_context_data_mut::<S, Q>(ctx);
    match b.querier.as_mut() {
        Some(q) => func(q),
        None => Err(VmError::uninitialized_context_data("querier")),
    }
}

#[cfg(feature = "iterator")]
pub(crate) fn with_iterator_from_context<'a, 'b, S, Q: 'b, F, T>(
    ctx: &'a mut Ctx,
    iterator_id: u32,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&'b mut Box<dyn StorageIterator + 'b>) -> VmResult<T>,
{
    let b = get_context_data_mut::<S, Q>(ctx);
    match b.iterators.get_mut(&iterator_id) {
        Some(iterator) => func(iterator),
        None => Err(VmError::iterator_does_not_exist(iterator_id)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backends::{compile, get_gas_left, set_gas_limit};
    use crate::errors::VmError;
    #[cfg(feature = "iterator")]
    use crate::testing::MockIterator;
    use crate::testing::{MockQuerier, MockStorage};
    use crate::traits::ReadonlyStorage;
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, HumanAddr, Never, QueryRequest,
    };
    use wasmer_runtime_core::{imports, typed_func::Func};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MS = MockStorage;
    type MQ = MockQuerier;

    // prepared data
    static INIT_KEY: &[u8] = b"foo";
    static INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    static INIT_ADDR: &str = "someone";
    static INIT_AMOUNT: u128 = 500;
    static INIT_DENOM: &str = "TOKEN";

    #[cfg(feature = "singlepass")]
    use crate::backends::singlepass::GAS_LIMIT;
    #[cfg(not(feature = "singlepass"))]
    const GAS_LIMIT: u64 = 10_000_000_000;

    fn make_instance() -> Box<WasmerInstance> {
        let module = compile(&CONTRACT).unwrap();
        // we need stubs for all required imports
        let import_obj = imports! {
            || { setup_context::<MockStorage, MockQuerier>(GAS_LIMIT) },
            "env" => {
                "db_read" => Func::new(|_a: i32| -> u32 { 0 }),
                "db_write" => Func::new(|_a: i32, _b: i32| {}),
                "db_remove" => Func::new(|_a: i32| {}),
                "db_scan" => Func::new(|_a: i32, _b: i32, _c: i32| -> u32 { 0 }),
                "db_next" => Func::new(|_a: u32| -> u32 { 0 }),
                "query_chain" => Func::new(|_a: i32| -> i32 { 0 }),
                "canonicalize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "humanize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
            },
        };
        let mut instance = Box::from(module.instantiate(&import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        set_wasmer_instance::<MS, MQ>(instance.context_mut(), Some(instance_ptr));

        instance
    }

    fn leave_default_data(ctx: &mut Ctx) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .expect("error setting value");
        let querier: MockQuerier<Never> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(ctx, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        // this creates an instance
        let mut instance = make_instance();
        let ctx = instance.context_mut();

        // empty data on start
        let (inits, initq) = move_out_of_context::<MS, MQ>(ctx);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(ctx);
        let (s, q) = move_out_of_context::<MS, MQ>(ctx);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).unwrap().0,
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = move_out_of_context::<MS, MQ>(ctx);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_limit(instance.as_mut(), gas_limit);
        get_gas_state::<MS, MQ>(instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Consume all the Gas that we allocated
        try_consume_gas::<MS, MQ>(context, 70).unwrap();
        try_consume_gas::<MS, MQ>(context, 4).unwrap();
        try_consume_gas::<MS, MQ>(context, 6).unwrap();
        try_consume_gas::<MS, MQ>(context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match try_consume_gas::<MS, MQ>(context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_limit(instance.as_mut(), gas_limit);
        get_gas_state::<MS, MQ>(instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Consume all the Gas that we allocated
        try_consume_gas::<MS, MQ>(context, 50).unwrap();
        try_consume_gas::<MS, MQ>(context, 4).unwrap();

        // consume 20 gas directly in wasmer
        let new_limit = get_gas_left(instance.as_mut()) - 20;
        set_gas_limit(instance.as_mut(), new_limit);

        let context = instance.context_mut();
        try_consume_gas::<MS, MQ>(context, 6).unwrap();
        try_consume_gas::<MS, MQ>(context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match try_consume_gas::<MS, MQ>(context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), true);
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // change
        set_storage_readonly::<MS, MQ>(ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), false);

        // still false
        set_storage_readonly::<MS, MQ>(ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), false);

        // change back
        set_storage_readonly::<MS, MQ>(ctx, true);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), true);
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
        leave_default_data(instance.context_mut());

        let ctx = instance.context_mut();
        let ptr = with_func_from_context::<MS, MQ, u32, u32, _, _>(ctx, "allocate", |alloc_func| {
            let ptr = alloc_func.call(10)?;
            Ok(ptr)
        })
        .unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn with_func_from_context_fails_for_missing_instance() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        // Clear context's wasmer_instance
        set_wasmer_instance::<MS, MQ>(instance.context_mut(), None);

        let ctx = instance.context_mut();
        let res = with_func_from_context::<MS, MQ, u32, u32, _, ()>(ctx, "allocate", |_func| {
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
        leave_default_data(instance.context_mut());

        let ctx = instance.context_mut();
        let res = with_func_from_context::<MS, MQ, u32, u32, _, ()>(ctx, "doesnt_exist", |_func| {
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
        leave_default_data(ctx);

        let (val, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            Ok(store.get(INIT_KEY).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            store.set(set_key, set_value).expect("error setting value");
            Ok(())
        })
        .unwrap();

        with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            assert_eq!(store.get(INIT_KEY).unwrap().0, Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).unwrap().0, Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        with_storage_from_context::<MS, MQ, _, ()>(ctx, |_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let res = with_querier_from_context::<MS, MQ, _, _>(ctx, |querier| {
            let req: QueryRequest<Never> = QueryRequest::Bank(BankQuery::AllBalances {
                address: HumanAddr::from(INIT_ADDR),
            });
            Ok(querier.raw_query(&to_vec(&req).unwrap())?)
        })
        .unwrap()
        .0
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
        leave_default_data(ctx);

        with_querier_from_context::<MS, MQ, _, ()>(ctx, |_querier| {
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
            assert!(iter.next().unwrap().0.is_none());
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
