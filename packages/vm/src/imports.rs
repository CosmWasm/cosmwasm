//! Import implementations

#[cfg(feature = "iterator")]
use std::convert::TryInto;
#[cfg(feature = "iterator")]
use std::mem;

#[cfg(feature = "iterator")]
use cosmwasm_std::StdResult;
use cosmwasm_std::{
    Api, ApiQuerierResponse, ApiSystemError, Binary, CanonicalAddr, HumanAddr, Querier,
    QuerierResponse, QueryRequest, Storage,
};
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};
use wasmer_runtime_core::vm::Ctx;

#[cfg(feature = "iterator")]
use crate::context::{add_iterator, with_iterator_from_context};
use crate::context::{with_querier_from_context, with_storage_from_context};
#[cfg(feature = "iterator")]
use crate::conversion::to_i32;
use crate::errors::{make_runtime_err, VmError};
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};
use crate::serde::{from_slice, to_vec};

/// A kibi (kilo binary)
static KI: usize = 1024;
/// Max key length for db_write (i.e. when VM reads from Wasm memory). Should match the
/// value for db_next (see DB_READ_KEY_BUFFER_LENGTH in packages/std/src/imports.rs)
static MAX_LENGTH_DB_KEY: usize = 64 * KI;
/// Max key length for db_write (i.e. when VM reads from Wasm memory). Should match the
/// value for db_read/db_next (see DB_READ_VALUE_BUFFER_LENGTH in packages/std/src/imports.rs)
static MAX_LENGTH_DB_VALUE: usize = 128 * KI;
/// Typically 20 (Cosmos SDK, Ethereum) or 32 (Nano, Substrate)
static MAX_LENGTH_CANONICAL_ADDRESS: usize = 32;
/// The maximum allowed size for bech32 (https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#bech32)
static MAX_LENGTH_HUMAN_ADDRESS: usize = 90;
static MAX_LENGTH_QUERY_CHAIN_REQUEST: usize = 64 * KI;

static SUCCESS: i32 = 0;
/// An unknown error occurred when writing to region
static ERROR_REGION_WRITE_UNKNOWN: i32 = -1_000_001;
/// Could not write to region because it is too small
static ERROR_REGION_WRITE_TOO_SMALL: i32 = -1_000_002;
/// An unknown error occurred when reading region
static ERROR_REGION_READ_UNKNOWN: i32 = -1_000_101;
/// The contract sent us a Region we're not willing to read because it is too big
static ERROR_REGION_READ_LENGTH_TOO_BIG: i32 = -1_000_102;
/// An unknown error when canonicalizing address
static ERROR_CANONICALIZE_UNKNOWN: i32 = -1_000_201;
/// The input address (human address) was invalid
static ERROR_CANONICALIZE_INVALID_INPUT: i32 = -1_000_202;
/// An unknonw error when humanizing address
static ERROR_HUMANIZE_UNKNOWN: i32 = -1_000_301;
/// Cannot serialize query response
static ERROR_QUERY_CHAIN_CANNOT_SERIALIZE_RESPONSE: i32 = -1_000_402;
/// Generic error - using context with no Storage attached
static ERROR_NO_CONTEXT_DATA: i32 = -1_000_501;
/// Generic error - An unknown error accessing the DB
static ERROR_DB_UNKNOWN: i32 = -1_000_502;

// The 2_xxx_xxx namespace is reserved for #[cfg(feature = "iterator")]

/// An unknown error in the db_scan implementation
#[cfg(feature = "iterator")]
static ERROR_SCAN_UNKNOWN: i32 = -2_000_001;
/// Invalid Order enum value passed into scan
#[cfg(feature = "iterator")]
static ERROR_SCAN_INVALID_ORDER: i32 = -2_000_002;
/// An unknown error in the db_next implementation
#[cfg(feature = "iterator")]
static ERROR_NEXT_UNKNOWN: i32 = -2_000_101;
/// Iterator pointer not registered
#[cfg(feature = "iterator")]
static ERROR_NEXT_INVALID_ITERATOR: i32 = -2_000_102;

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_read<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let value: Option<Vec<u8>> = match with_storage_from_context::<S, Q, _, _>(ctx, |store| {
        store
            .get(&key)
            .or_else(|_| make_runtime_err("Error reading from backend"))
    }) {
        Ok(v) => v,
        Err(VmError::UninitializedContextData { .. }) => return ERROR_NO_CONTEXT_DATA,
        Err(_) => return ERROR_DB_UNKNOWN,
    };
    match value {
        Some(buf) => match write_region(ctx, value_ptr, &buf) {
            Ok(()) => SUCCESS,
            Err(VmError::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        None => SUCCESS,
    }
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let value = match read_region(ctx, value_ptr, MAX_LENGTH_DB_VALUE) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    match with_storage_from_context::<S, Q, _, ()>(ctx, |store| {
        store
            .set(&key, &value)
            .or_else(|_| make_runtime_err("Error setting database value in backend"))
    }) {
        Ok(_) => SUCCESS,
        Err(VmError::UninitializedContextData { .. }) => ERROR_NO_CONTEXT_DATA,
        Err(_) => ERROR_DB_UNKNOWN,
    }
}

pub fn do_remove<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    match with_storage_from_context::<S, Q, _, ()>(ctx, |store| {
        store
            .remove(&key)
            .or_else(|_| make_runtime_err("Error removing database key from backend"))
    }) {
        Ok(_) => SUCCESS,
        Err(VmError::UninitializedContextData { .. }) => ERROR_NO_CONTEXT_DATA,
        Err(_) => ERROR_DB_UNKNOWN,
    }
}

pub fn do_canonicalize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    human_ptr: u32,
    canonical_ptr: u32,
) -> i32 {
    let human_data = match read_region(ctx, human_ptr, MAX_LENGTH_HUMAN_ADDRESS) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let human = match String::from_utf8(human_data) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return ERROR_CANONICALIZE_INVALID_INPUT,
    };
    match api.canonical_address(&human) {
        Ok(canon) => match write_region(ctx, canonical_ptr, canon.as_slice()) {
            Ok(()) => SUCCESS,
            Err(VmError::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_CANONICALIZE_UNKNOWN,
    }
}

pub fn do_humanize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    canonical_ptr: u32,
    human_ptr: u32,
) -> i32 {
    let canonical = match read_region(ctx, canonical_ptr, MAX_LENGTH_CANONICAL_ADDRESS) {
        Ok(data) => Binary(data),
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    match api.human_address(&CanonicalAddr(canonical)) {
        Ok(human) => match write_region(ctx, human_ptr, human.as_str().as_bytes()) {
            Ok(()) => SUCCESS,
            Err(VmError::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_HUMANIZE_UNKNOWN,
    }
}

pub fn do_query_chain<A: Api, S: Storage, Q: Querier>(
    _api: A,
    ctx: &mut Ctx,
    request_ptr: u32,
    response_ptr: u32,
) -> i32 {
    let request = match read_region(ctx, request_ptr, MAX_LENGTH_QUERY_CHAIN_REQUEST) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };

    let res = match from_slice::<QueryRequest>(&request) {
        // if we parse, try to execute the query
        Ok(parsed) => {
            let qr: QuerierResponse =
                with_querier_from_context::<S, Q, _, _>(ctx, |querier: &Q| querier.query(&parsed));
            qr
        }
        // otherwise, return the InvalidRequest error as ApiSystemError
        Err(err) => Err(ApiSystemError::InvalidRequest {
            error: err.to_string(),
        }),
    };

    let api_res: ApiQuerierResponse = res.into();

    match to_vec(&api_res) {
        Ok(serialized) => match write_region(ctx, response_ptr, &serialized) {
            Ok(()) => SUCCESS,
            Err(VmError::RegionTooSmallErr { .. }) => ERROR_REGION_WRITE_TOO_SMALL,
            Err(_) => ERROR_REGION_WRITE_UNKNOWN,
        },
        Err(_) => ERROR_QUERY_CHAIN_CANNOT_SERIALIZE_RESPONSE,
    }
}

#[cfg(feature = "iterator")]
pub fn do_scan<S: Storage + 'static, Q: Querier>(
    ctx: &mut Ctx,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> i32 {
    let start = match maybe_read_region(ctx, start_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let end = match maybe_read_region(ctx, end_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return ERROR_REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return ERROR_REGION_READ_UNKNOWN,
    };
    let order: Order = match order.try_into() {
        Ok(o) => o,
        Err(_) => return ERROR_SCAN_INVALID_ORDER,
    };
    let range_result = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
        let iter = match store.range(start.as_deref(), end.as_deref(), order) {
            Ok(iter) => iter,
            Err(_) => return make_runtime_err("An error occurred in range call"),
        };

        // Unsafe: I know the iterator will be deallocated before the storage as I control the lifetime below
        // But there is no way for the compiler to know. So... let's just lie to the compiler a little bit.
        let live_forever: Box<dyn Iterator<Item = StdResult<KV>> + 'static> =
            unsafe { mem::transmute(iter) };
        Ok(live_forever)
    });

    match range_result {
        Ok(iterator) => {
            let new_id = add_iterator::<S, Q>(ctx, iterator);
            match to_i32(new_id) {
                Ok(new_id_signed) => new_id_signed,
                Err(_) => ERROR_SCAN_UNKNOWN,
            }
        }
        Err(VmError::UninitializedContextData { .. }) => ERROR_NO_CONTEXT_DATA,
        Err(_) => ERROR_SCAN_UNKNOWN,
    }
}

#[cfg(feature = "iterator")]
pub fn do_next<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    iterator_id: u32,
    key_ptr: u32,
    value_ptr: u32,
) -> i32 {
    let item =
        match with_iterator_from_context::<S, Q, _, _>(ctx, iterator_id, |iter| Ok(iter.next())) {
            Ok(i) => i,
            Err(VmError::UninitializedContextData { .. }) => return ERROR_NO_CONTEXT_DATA,
            Err(_) => return ERROR_NEXT_INVALID_ITERATOR,
        };

    // prepare return values
    let (key, value) = match item {
        Some(Ok(item)) => item,
        Some(Err(_)) => return ERROR_NEXT_UNKNOWN,
        None => return SUCCESS, // Return early without writing key. Empty key will later be treated as _no more element_.
    };

    match write_region(ctx, key_ptr, &key) {
        Ok(()) => (),
        Err(VmError::RegionTooSmallErr { .. }) => return ERROR_REGION_WRITE_TOO_SMALL,
        Err(_) => return ERROR_REGION_WRITE_UNKNOWN,
    };
    match write_region(ctx, value_ptr, &value) {
        Ok(()) => (),
        Err(VmError::RegionTooSmallErr { .. }) => return ERROR_REGION_WRITE_TOO_SMALL,
        Err(_) => return ERROR_REGION_WRITE_UNKNOWN,
    };
    SUCCESS
}

#[cfg(test)]
#[cfg(feature = "iterator")]
mod test {
    use super::*;
    use cosmwasm_std::testing::{MockQuerier, MockStorage};
    use cosmwasm_std::{coins, HumanAddr};
    use wasmer_runtime_core::{imports, instance::Instance, typed_func::Func};

    use crate::backends::compile;
    use crate::context::{move_into_context, setup_context};
    #[cfg(feature = "iterator")]
    use crate::conversion::to_u32;

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthand for function generics below
    type S = MockStorage;
    type Q = MockQuerier;

    // prepared data
    static KEY1: &[u8] = b"ant";
    static VALUE1: &[u8] = b"insect";
    static KEY2: &[u8] = b"tree";
    static VALUE2: &[u8] = b"plant";

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
        storage.set(KEY1, VALUE1).expect("error setting");
        storage.set(KEY2, VALUE2).expect("error setting");
        let querier =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(ctx, storage, querier);
    }

    fn write_data(wasmer_instance: &mut Instance, data: &[u8]) -> u32 {
        let allocate: Func<u32, u32> = wasmer_instance
            .func("allocate")
            .expect("error getting function");
        let region_ptr = allocate
            .call(data.len() as u32)
            .expect("error calling allocate");
        write_region(wasmer_instance.context_mut(), region_ptr, data).expect("error writing");
        region_ptr
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let id = to_u32(do_scan::<S, Q>(ctx, 0, 0, Order::Ascending.into()))
            .expect("ID must not be negative");
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_descending_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let id = to_u32(do_scan::<S, Q>(ctx, 0, 0, Order::Descending.into()))
            .expect("ID must not be negative");
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_bound_works() {
        let mut instance = make_instance();

        let start = write_data(&mut instance, b"anna");
        let end = write_data(&mut instance, b"bert");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = to_u32(do_scan::<S, Q>(ctx, start, end, Order::Ascending.into()))
            .expect("ID must not be negative");

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_multiple_iterators() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // unbounded, ascending and descending
        let id1 = to_u32(do_scan::<S, Q>(ctx, 0, 0, Order::Ascending.into()))
            .expect("ID must not be negative");
        let id2 = to_u32(do_scan::<S, Q>(ctx, 0, 0, Order::Descending.into()))
            .expect("ID must not be negative");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // first item, first iterator
        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        // second item, first iterator
        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // first item, second iterator
        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // end, first iterator
        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());

        // second item, second iterator
        let item =
            with_iterator_from_context::<S, Q, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));
    }
}
