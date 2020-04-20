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

mod errors {
    /// Success
    pub static NONE: i32 = 0;
    /// An unknown error occurred when writing to region
    pub static REGION_WRITE_UNKNOWN: i32 = -1_000_000;
    /// Could not write to region because it is too small
    pub static REGION_WRITE_TOO_SMALL: i32 = -1_000_001;
    /// An unknown error occurred when reading region
    pub static REGION_READ_UNKNOWN: i32 = -1_000_100;
    /// The contract sent us a Region we're not willing to read because it is too big
    pub static REGION_READ_LENGTH_TOO_BIG: i32 = -1_000_101;

    // unused block (-1_000_2xx)
    // unused block (-1_000_3xx)
    // unused block (-1_000_4xx)

    /// Generic error - using context with no Storage attached
    pub static NO_CONTEXT_DATA: i32 = -1_000_500;
    /// Generic error - An unknown error accessing the DB
    pub static DB_UNKNOWN: i32 = -1_000_501;

    /// db_read errors (-1_001_0xx)
    pub mod read {
        // pub static UNKNOWN: i32 = -1_001_000;
        /// The given key does not exist in storage
        pub static KEY_DOES_NOT_EXIST: i32 = -1_001_001;
    }

    /// db_write errors (-1_001_1xx)
    /// db_remove errors (-1_001_2xx)

    /// canonicalize_address errors (-1_002_0xx)
    pub mod canonicalize {
        /// An unknown error when canonicalizing address
        pub static UNKNOWN: i32 = -1_002_000;
        /// The input address (human address) was invalid
        pub static INVALID_INPUT: i32 = -1_002_001;
    }

    /// humanize_address errors (-1_002_1xx)
    pub mod humanize {
        /// An unknonw error when humanizing address
        pub static UNKNOWN: i32 = -1_002_100;
    }

    /// query_chain errors (-1_003_0xx)
    pub mod query_chain {
        /// An unknown error in query_chain
        // pub static UNKNOWN: i32 = -1_003_000;
        /// Cannot serialize query response
        pub static CANNOT_SERIALIZE_RESPONSE: i32 = -1_003_001;
    }

    // The -2_xxx_xxx namespace is reserved for #[cfg(feature = "iterator")]

    /// db_scan errors (-2_000_0xx)
    #[cfg(feature = "iterator")]
    pub mod scan {
        /// An unknown error in the db_scan implementation
        pub static UNKNOWN: i32 = -2_000_001;
        /// Invalid Order enum value passed into scan
        pub static INVALID_ORDER: i32 = -2_000_002;
    }

    /// db_next errors (-2_000_1xx)
    #[cfg(feature = "iterator")]
    pub mod next {
        /// An unknown error in the db_next implementation
        pub static UNKNOWN: i32 = -2_000_101;
        /// Iterator pointer not registered
        pub static INVALID_ITERATOR: i32 = -2_000_102;
    }
}

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_read<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    let value: Option<Vec<u8>> = match with_storage_from_context::<S, Q, _, _>(ctx, |store| {
        store
            .get(&key)
            .or_else(|_| make_runtime_err("Error reading from backend"))
    }) {
        Ok(v) => v,
        Err(VmError::UninitializedContextData { .. }) => return errors::NO_CONTEXT_DATA,
        Err(_) => return errors::DB_UNKNOWN,
    };
    match value {
        Some(buf) => match write_region(ctx, value_ptr, &buf) {
            Ok(()) => errors::NONE,
            Err(VmError::RegionTooSmallErr { .. }) => errors::REGION_WRITE_TOO_SMALL,
            Err(_) => errors::REGION_WRITE_UNKNOWN,
        },
        None => errors::read::KEY_DOES_NOT_EXIST,
    }
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32, value_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    let value = match read_region(ctx, value_ptr, MAX_LENGTH_DB_VALUE) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    match with_storage_from_context::<S, Q, _, ()>(ctx, |store| {
        store
            .set(&key, &value)
            .or_else(|_| make_runtime_err("Error setting database value in backend"))
    }) {
        Ok(_) => errors::NONE,
        Err(VmError::UninitializedContextData { .. }) => errors::NO_CONTEXT_DATA,
        Err(_) => errors::DB_UNKNOWN,
    }
}

pub fn do_remove<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32) -> i32 {
    let key = match read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    match with_storage_from_context::<S, Q, _, ()>(ctx, |store| {
        store
            .remove(&key)
            .or_else(|_| make_runtime_err("Error removing database key from backend"))
    }) {
        Ok(_) => errors::NONE,
        Err(VmError::UninitializedContextData { .. }) => errors::NO_CONTEXT_DATA,
        Err(_) => errors::DB_UNKNOWN,
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
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    let human = match String::from_utf8(human_data) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return errors::canonicalize::INVALID_INPUT,
    };
    match api.canonical_address(&human) {
        Ok(canon) => match write_region(ctx, canonical_ptr, canon.as_slice()) {
            Ok(()) => errors::NONE,
            Err(VmError::RegionTooSmallErr { .. }) => errors::REGION_WRITE_TOO_SMALL,
            Err(_) => errors::REGION_WRITE_UNKNOWN,
        },
        Err(_) => errors::canonicalize::UNKNOWN,
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
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    match api.human_address(&CanonicalAddr(canonical)) {
        Ok(human) => match write_region(ctx, human_ptr, human.as_str().as_bytes()) {
            Ok(()) => errors::NONE,
            Err(VmError::RegionTooSmallErr { .. }) => errors::REGION_WRITE_TOO_SMALL,
            Err(_) => errors::REGION_WRITE_UNKNOWN,
        },
        Err(_) => errors::humanize::UNKNOWN,
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
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
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
            Ok(()) => errors::NONE,
            Err(VmError::RegionTooSmallErr { .. }) => errors::REGION_WRITE_TOO_SMALL,
            Err(_) => errors::REGION_WRITE_UNKNOWN,
        },
        Err(_) => errors::query_chain::CANNOT_SERIALIZE_RESPONSE,
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
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    let end = match maybe_read_region(ctx, end_ptr, MAX_LENGTH_DB_KEY) {
        Ok(data) => data,
        Err(VmError::RegionLengthTooBigErr { .. }) => return errors::REGION_READ_LENGTH_TOO_BIG,
        Err(_) => return errors::REGION_READ_UNKNOWN,
    };
    let order: Order = match order.try_into() {
        Ok(o) => o,
        Err(_) => return errors::scan::INVALID_ORDER,
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
                Err(_) => errors::scan::UNKNOWN,
            }
        }
        Err(VmError::UninitializedContextData { .. }) => errors::NO_CONTEXT_DATA,
        Err(_) => errors::scan::UNKNOWN,
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
            Err(VmError::UninitializedContextData { .. }) => return errors::NO_CONTEXT_DATA,
            Err(_) => return errors::next::INVALID_ITERATOR,
        };

    // prepare return values
    let (key, value) = match item {
        Some(Ok(item)) => item,
        Some(Err(_)) => return errors::next::UNKNOWN,
        None => return errors::NONE, // Return early without writing key. Empty key will later be treated as _no more element_.
    };

    match write_region(ctx, key_ptr, &key) {
        Ok(()) => (),
        Err(VmError::RegionTooSmallErr { .. }) => return errors::REGION_WRITE_TOO_SMALL,
        Err(_) => return errors::REGION_WRITE_UNKNOWN,
    };
    match write_region(ctx, value_ptr, &value) {
        Ok(()) => (),
        Err(VmError::RegionTooSmallErr { .. }) => return errors::REGION_WRITE_TOO_SMALL,
        Err(_) => return errors::REGION_WRITE_UNKNOWN,
    };
    errors::NONE
}

#[cfg(test)]
#[cfg(feature = "iterator")]
mod test {
    use super::*;
    use cosmwasm_std::testing::{MockQuerier, MockStorage};
    use cosmwasm_std::{coins, HumanAddr, ReadonlyStorage};
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

    fn create_empty(wasmer_instance: &mut Instance, size: u32) -> u32 {
        let allocate: Func<u32, u32> = wasmer_instance
            .func("allocate")
            .expect("error getting function");
        let region_ptr = allocate.call(size).expect("error calling allocate");
        region_ptr
    }

    #[test]
    fn do_read_works() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::NONE);
        assert_eq!(read_region(ctx, value_ptr, 500).unwrap(), VALUE1);
    }

    #[test]
    fn do_read_works_for_non_existent_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"I do not exist in storage");
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::read::KEY_DOES_NOT_EXIST);
        assert!(read_region(ctx, value_ptr, 500).unwrap().is_empty());
    }

    #[test]
    fn do_read_fails_for_large_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, &vec![7u8; 300 * 1024]);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::REGION_READ_LENGTH_TOO_BIG);
        assert!(read_region(ctx, value_ptr, 500).unwrap().is_empty());
    }

    #[test]
    fn do_read_fails_for_small_result_region() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = create_empty(&mut instance, 3);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::REGION_WRITE_TOO_SMALL);
        assert!(read_region(ctx, value_ptr, 500).unwrap().is_empty());
    }

    #[test]
    fn do_write_works() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"new value");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::NONE);

        let val = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(b"new storage key").expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(b"new value".to_vec()));
    }

    #[test]
    fn do_write_works_for_empty_value() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::NONE);

        let val = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(b"new storage key").expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(b"".to_vec()));
    }

    #[test]
    fn do_write_fails_for_large_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, &vec![4u8; 300 * 1024]);
        let value_ptr = write_data(&mut instance, b"new value");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_remove_works() {
        let mut instance = make_instance();

        let existing_key = KEY1;
        let key_ptr = write_data(&mut instance, existing_key);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_remove::<S, Q>(ctx, key_ptr);
        assert_eq!(result, errors::NONE);

        let value = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(existing_key).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_remove_works_for_non_existent_key() {
        let mut instance = make_instance();

        let non_existent_key = b"I do not exist";
        let key_ptr = write_data(&mut instance, non_existent_key);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_remove::<S, Q>(ctx, key_ptr);
        // Note: right now we cannot differnetiate between an existent and a non-existent key
        assert_eq!(result, errors::NONE);

        let value = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(non_existent_key).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_remove_fails_for_large_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, &vec![26u8; 300 * 1024]);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_remove::<S, Q>(ctx, key_ptr);
        assert_eq!(result, errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_write_fails_for_large_value() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, &vec![5u8; 300 * 1024]);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_write_can_override() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = write_data(&mut instance, VALUE2);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<S, Q>(ctx, key_ptr, value_ptr);
        assert_eq!(result, errors::NONE);

        let val = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
            Ok(store.get(KEY1).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(VALUE2.to_vec()));
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
