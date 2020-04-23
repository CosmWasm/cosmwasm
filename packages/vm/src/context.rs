//! Internal details to be used by instance.rs only
#[cfg(feature = "iterator")]
use std::collections::HashMap;
#[cfg(feature = "iterator")]
use std::convert::TryInto;
use std::ffi::c_void;

use wasmer_runtime_core::vm::Ctx;

use cosmwasm_std::{Querier, Storage, SystemError};
#[cfg(feature = "iterator")]
use cosmwasm_std::{StdResult, KV};

#[cfg(feature = "iterator")]
use crate::errors::IteratorDoesNotExist;
use crate::errors::{UninitializedContextData, VmResult};

/** context data **/

struct ContextData<S: Storage, Q: Querier> {
    storage: Option<S>,
    querier: Option<Q>,
    #[cfg(feature = "iterator")]
    iterators: HashMap<u32, Box<dyn Iterator<Item = StdResult<KV>>>>,
}

pub fn setup_context<S: Storage, Q: Querier>() -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_context_data::<S, Q>(),
        destroy_unmanaged_context_data::<S, Q>,
    )
}

fn create_unmanaged_context_data<S: Storage, Q: Querier>() -> *mut c_void {
    let data = ContextData::<S, Q> {
        storage: None,
        querier: None,
        #[cfg(feature = "iterator")]
        iterators: HashMap::new(),
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
fn get_context_data<S: Storage, Q: Querier>(ctx: &mut Ctx) -> &mut ContextData<S, Q> {
    let owned = unsafe {
        Box::from_raw(ctx.data as *mut ContextData<S, Q>) // obtain ownership
    };
    Box::leak(owned) // give up ownership
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
    let mut b = get_context_data::<S, Q>(source);
    // Destroy all existing iterators which are (in contrast to the storage)
    // not reused between different instances.
    destroy_iterators(&mut b);
    (b.storage.take(), b.querier.take())
}

/// Moves owned instances of storage and querier into the context.
/// Should be followed by exactly one call to move_out_of_context when the instance is finished.
pub(crate) fn move_into_context<S: Storage, Q: Querier>(target: &mut Ctx, storage: S, querier: Q) {
    let b = get_context_data::<S, Q>(target);
    b.storage = Some(storage);
    b.querier = Some(querier);
}

/// Add the iterator to the context's data. A new ID is assigned and returned.
/// IDs are guaranteed to be in the range [0, 2**31-1], i.e. fit in the non-negative part if type i32.
#[cfg(feature = "iterator")]
#[must_use = "without the returned iterator ID, the iterator cannot be accessed"]
pub fn add_iterator<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    iter: Box<dyn Iterator<Item = StdResult<KV>>>,
) -> u32 {
    let b = get_context_data::<S, Q>(ctx);
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

pub(crate) fn with_storage_from_context<S, Q, F, T>(ctx: &mut Ctx, mut func: F) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnMut(&mut S) -> VmResult<T>,
{
    let b = get_context_data::<S, Q>(ctx);
    let mut storage = b.storage.take();
    let res = match &mut storage {
        Some(data) => func(data),
        None => UninitializedContextData { kind: "storage" }.fail(),
    };
    b.storage = storage;
    res
}

pub(crate) fn with_querier_from_context<S, Q, F, T>(
    ctx: &mut Ctx,
    mut func: F,
) -> Result<T, SystemError>
where
    S: Storage,
    Q: Querier,
    F: FnMut(&Q) -> Result<T, SystemError>,
{
    let b = get_context_data::<S, Q>(ctx);
    let querier = b.querier.take();
    let res = match &querier {
        Some(q) => func(q),
        None => Err(SystemError::Unknown {}),
    };
    b.querier = querier;
    res
}

#[cfg(feature = "iterator")]
pub(crate) fn with_iterator_from_context<S, Q, F, T>(
    ctx: &mut Ctx,
    iterator_id: u32,
    mut func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnMut(&mut dyn Iterator<Item = StdResult<KV>>) -> VmResult<T>,
{
    let b = get_context_data::<S, Q>(ctx);
    let iter = b.iterators.remove(&iterator_id);
    match iter {
        Some(mut data) => {
            let res = func(&mut data);
            b.iterators.insert(iterator_id, data);
            res
        }
        None => IteratorDoesNotExist { id: iterator_id }.fail(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backends::compile;
    #[cfg(feature = "iterator")]
    use crate::errors::VmError;
    use cosmwasm_std::testing::{MockQuerier, MockStorage};
    use cosmwasm_std::{
        coin, coins, from_binary, AllBalanceResponse, BankQuery, HumanAddr, QueryRequest,
        ReadonlyStorage,
    };
    use wasmer_runtime_core::{imports, instance::Instance, typed_func::Func};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthand for function generics below
    type S = MockStorage;
    type Q = MockQuerier;

    // prepared data
    static INIT_KEY: &[u8] = b"foo";
    static INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    static INIT_ADDR: &str = "someone";
    static INIT_AMOUNT: u128 = 500;
    static INIT_DENOM: &str = "TOKEN";

    fn make_instance() -> Instance {
        let module = compile(&CONTRACT).unwrap();
        // we need stubs for all required imports
        let import_obj = imports! {
            || { setup_context::<MockStorage, MockQuerier>() },
            "env" => {
                "db_read" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "db_write" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "db_remove" => Func::new(|_a: i32| -> i32 { 0 }),
                "db_scan" => Func::new(|_a: i32, _b: i32, _c: i32| -> i32 { 0 }),
                "db_next" => Func::new(|_a: u32, _b: i32, _c: i32| -> i32 { 0 }),
                "query_chain" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "canonicalize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "humanize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
            },
        };
        let instance = module.instantiate(&import_obj).unwrap();
        instance
    }

    fn leave_default_data(ctx: &mut Ctx) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .expect("error setting value");
        let querier =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(ctx, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        // this creates an instance
        let mut instance = make_instance();
        let ctx = instance.context_mut();

        // empty data on start
        let (inits, initq) = move_out_of_context::<S, Q>(ctx);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(ctx);
        let (s, q) = move_out_of_context::<S, Q>(ctx);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(s.unwrap().get(INIT_KEY).unwrap(), Some(INIT_VALUE.to_vec()));

        // now is empty again
        let (ends, endq) = move_out_of_context::<S, Q>(ctx);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn add_iterator_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        assert_eq!(get_context_data::<S, Q>(ctx).iterators.len(), 0);
        let id1 = add_iterator::<S, Q>(ctx, Box::new(std::iter::empty()));
        let id2 = add_iterator::<S, Q>(ctx, Box::new(std::iter::empty()));
        let id3 = add_iterator::<S, Q>(ctx, Box::new(std::iter::empty()));
        assert_eq!(get_context_data::<S, Q>(ctx).iterators.len(), 3);
        assert!(get_context_data::<S, Q>(ctx).iterators.contains_key(&id1));
        assert!(get_context_data::<S, Q>(ctx).iterators.contains_key(&id2));
        assert!(get_context_data::<S, Q>(ctx).iterators.contains_key(&id3));
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let val = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(INIT_KEY).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            store.set(set_key, set_value).expect("error setting value");
            Ok(())
        })
        .unwrap();

        with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            assert_eq!(store.get(INIT_KEY).unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).unwrap(), Some(set_value.to_vec()));
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

        with_storage_from_context::<S, Q, _, ()>(ctx, |_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let res = with_querier_from_context::<S, Q, _, _>(ctx, |querier| {
            let req = QueryRequest::Bank(BankQuery::AllBalances {
                address: HumanAddr::from(INIT_ADDR),
            });
            querier.query(&req)
        })
        .unwrap()
        .unwrap();
        let balance: AllBalanceResponse = from_binary(&res).unwrap();

        assert_eq!(balance.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    fn with_querier_from_context_parse_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);
        let contract = HumanAddr::from(INIT_ADDR);

        let balance = with_querier_from_context::<S, Q, _, _>(ctx, |querier| {
            Ok(querier.query_balance(&contract, INIT_DENOM))
        })
        .unwrap()
        .unwrap();
        assert_eq!(balance.amount, coin(INIT_AMOUNT, INIT_DENOM));

        let balance = with_querier_from_context::<S, Q, _, _>(ctx, |querier| {
            Ok(querier.query_balance(&contract, "foo"))
        })
        .unwrap()
        .unwrap();
        assert_eq!(balance.amount, coin(0, "foo"));
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_querier_from_context_handles_panics() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        with_querier_from_context::<S, Q, _, ()>(ctx, |_querier| {
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

        let id = add_iterator::<S, Q>(ctx, Box::new(std::iter::empty()));
        with_iterator_from_context::<S, Q, _, ()>(ctx, id, |iter| {
            assert!(iter.next().is_none());
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
        let miss = with_iterator_from_context::<S, Q, _, ()>(ctx, missing_id, |_iter| {
            panic!("this should not be called");
        });
        match miss {
            Ok(_) => panic!("Expected error"),
            Err(VmError::IteratorDoesNotExist { id, .. }) => assert_eq!(id, missing_id),
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
