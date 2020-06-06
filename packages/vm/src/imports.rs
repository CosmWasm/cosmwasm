//! Import implementations

#[cfg(feature = "iterator")]
use std::convert::TryInto;

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr};
use wasmer_runtime_core::vm::Ctx;

#[cfg(feature = "iterator")]
use crate::context::{add_iterator, with_iterator_from_context};
use crate::context::{is_storage_readonly, with_querier_from_context, with_storage_from_context};
#[cfg(feature = "iterator")]
use crate::conversion::to_i32;
use crate::errors::{VmError, VmResult};
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};
use crate::serde::to_vec;
use crate::traits::{Api, Querier, Storage};

/// A kibi (kilo binary)
static KI: usize = 1024;
/// Max key length for db_write (i.e. when VM reads from Wasm memory). Should match the
/// value for db_next (see DB_READ_KEY_BUFFER_LENGTH in packages/std/src/imports.rs)
static MAX_LENGTH_DB_KEY: usize = 64 * KI;
/// Max key length for db_write (i.e. when VM reads from Wasm memory). Should match the
/// value for db_read/db_next (see DB_READ_VALUE_BUFFER_LENGTH in packages/std/src/imports.rs)
static MAX_LENGTH_DB_VALUE: usize = 128 * KI;
/// Typically 20 (Cosmos SDK, Ethereum) or 32 (Nano, Substrate)
const MAX_LENGTH_CANONICAL_ADDRESS: usize = 32;
/// The maximum allowed size for bech32 (https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#bech32)
const MAX_LENGTH_HUMAN_ADDRESS: usize = 90;
static MAX_LENGTH_QUERY_CHAIN_REQUEST: usize = 64 * KI;

// TODO convert these numbers to a single enum.
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
        /// The input address (human address) was invalid
        pub static INVALID_INPUT: i32 = -1_002_001;
    }

    // /// humanize_address errors (-1_002_1xx)
    // pub mod humanize {
    // }

    /// query_chain errors (-1_003_0xx)
    pub mod query_chain {
        /// Cannot serialize query response
        pub static CANNOT_SERIALIZE_RESPONSE: i32 = -1_003_001;
    }

    // The -2_xxx_xxx namespace is reserved for #[cfg(feature = "iterator")]

    /// db_scan errors (-2_000_0xx)
    #[cfg(feature = "iterator")]
    pub mod scan {
        /// Invalid Order enum value passed into scan
        pub static INVALID_ORDER: i32 = -2_000_002;
    }

    /// db_next errors (-2_000_1xx)
    #[cfg(feature = "iterator")]
    pub mod next {
        /// Iterator with the given ID is not registered
        pub static ITERATOR_DOES_NOT_EXIST: i32 = -2_000_102;
    }
}

/// This macro wraps the read_region function for the purposes of the functions below which need to report errors
/// in its operation to the caller in the WASM runtime.
/// On success, the read data is returned from the expression.
/// On failure, an error number is wrapped in `Ok` and returned from the function.
macro_rules! read_region {
    ($ctx: expr, $ptr: expr, $length: expr) => {
        match read_region($ctx, $ptr, $length) {
            Ok(data) => data,
            Err(err) => {
                return Ok(match err {
                    VmError::RegionLengthTooBig { .. } => errors::REGION_READ_LENGTH_TOO_BIG,
                    _ => errors::REGION_READ_UNKNOWN,
                })
            }
        }
    };
}

#[cfg(feature = "iterator")]
/// This macro wraps the maybe_read_region function for the purposes of the functions below which need to report errors
/// in its operation to the caller in the WASM runtime.
/// On success, the optionally read data is returned from the expression.
/// On failure, an error number is wrapped in `Ok` and returned from the function.
macro_rules! maybe_read_region {
    ($ctx: expr, $ptr: expr, $length: expr) => {
        match maybe_read_region($ctx, $ptr, $length) {
            Ok(data) => data,
            Err(err) => {
                return Ok(match err {
                    VmError::RegionLengthTooBig { .. } => errors::REGION_READ_LENGTH_TOO_BIG,
                    _ => errors::REGION_READ_UNKNOWN,
                })
            }
        }
    };
}

/// This macro wraps the write_region function for the purposes of the functions below which need to report errors
/// in its operation to the caller in the WASM runtime.
/// On success, `errors::NONE` is returned from the expression.
/// On failure, an error number is wrapped in `Ok` and returned from the function.
macro_rules! write_region {
    ($ctx: expr, $ptr: expr, $buffer: expr) => {
        match write_region($ctx, $ptr, $buffer) {
            Ok(()) => errors::NONE,
            Err(err) => {
                return Ok(match err {
                    VmError::RegionTooSmall { .. } => errors::REGION_WRITE_TOO_SMALL,
                    _ => errors::REGION_WRITE_UNKNOWN,
                })
            }
        }
    };
}

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_read<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<i32> {
    let key = read_region!(ctx, key_ptr, MAX_LENGTH_DB_KEY);
    // `Ok(expr?)` used to convert the error variant.
    let value: Option<Vec<u8>> =
        with_storage_from_context::<S, Q, _, _>(ctx, |store| Ok(store.get(&key)?))?;
    Ok(match value {
        Some(buf) => write_region!(ctx, value_ptr, &buf),
        None => errors::read::KEY_DOES_NOT_EXIST,
    })
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<i32> {
    if is_storage_readonly::<S, Q>(ctx) {
        return Err(VmError::write_access_denied());
    }

    let key = read_region!(ctx, key_ptr, MAX_LENGTH_DB_KEY);
    let value = read_region!(ctx, value_ptr, MAX_LENGTH_DB_VALUE);
    with_storage_from_context::<S, Q, _, ()>(ctx, |store| Ok(store.set(&key, &value)?))
        .and(Ok(errors::NONE))
}

pub fn do_remove<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32) -> VmResult<i32> {
    if is_storage_readonly::<S, Q>(ctx) {
        return Err(VmError::write_access_denied());
    }

    let key = read_region!(ctx, key_ptr, MAX_LENGTH_DB_KEY);
    with_storage_from_context::<S, Q, _, ()>(ctx, |store| Ok(store.remove(&key)?))
        .and(Ok(errors::NONE))
}

pub fn do_canonicalize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    human_ptr: u32,
    canonical_ptr: u32,
) -> VmResult<i32> {
    let human_data = read_region!(ctx, human_ptr, MAX_LENGTH_HUMAN_ADDRESS);
    let human = match String::from_utf8(human_data) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return Ok(errors::canonicalize::INVALID_INPUT),
    };
    let canon = api.canonical_address(&human)?;
    Ok(write_region!(ctx, canonical_ptr, canon.as_slice()))
}

pub fn do_humanize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    canonical_ptr: u32,
    human_ptr: u32,
) -> VmResult<i32> {
    let canonical = Binary(read_region!(
        ctx,
        canonical_ptr,
        MAX_LENGTH_CANONICAL_ADDRESS
    ));
    let human = api.human_address(&CanonicalAddr(canonical))?;
    Ok(write_region!(ctx, human_ptr, human.as_str().as_bytes()))
}

pub fn do_query_chain<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    request_ptr: u32,
    response_ptr: u32,
) -> VmResult<i32> {
    let request = read_region!(ctx, request_ptr, MAX_LENGTH_QUERY_CHAIN_REQUEST);

    let res =
        with_querier_from_context::<S, Q, _, _>(ctx, |querier| Ok(querier.raw_query(&request)?))?;

    Ok(match to_vec(&res) {
        Ok(serialized) => write_region!(ctx, response_ptr, &serialized),
        Err(_) => errors::query_chain::CANNOT_SERIALIZE_RESPONSE,
    })
}

#[cfg(feature = "iterator")]
pub fn do_scan<S: Storage + 'static, Q: Querier>(
    ctx: &mut Ctx,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> VmResult<i32> {
    let start = maybe_read_region!(ctx, start_ptr, MAX_LENGTH_DB_KEY);
    let end = maybe_read_region!(ctx, end_ptr, MAX_LENGTH_DB_KEY);
    let order: Order = match order.try_into() {
        Ok(order) => order,
        Err(_) => return Ok(errors::scan::INVALID_ORDER),
    };
    let iterator = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
        Ok(store.range(start.as_deref(), end.as_deref(), order)?)
    })?;

    let new_id = add_iterator::<S, Q>(ctx, iterator);
    to_i32(new_id)
}

#[cfg(feature = "iterator")]
pub fn do_next<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    iterator_id: u32,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<i32> {
    // This always succeeds but `?` is cheaper  and more future-proof than `unwrap` :D
    let result = with_iterator_from_context::<S, Q, _, _>(ctx, iterator_id, |iter| Ok(iter.next()));
    // This error variant is caused by user input, so we let the user know about it using an explicit error code.
    if let Err(VmError::IteratorDoesNotExist { .. }) = result {
        return Ok(errors::next::ITERATOR_DOES_NOT_EXIST);
    }
    let item = result?;

    // Prepare return values. Both key and value are Options and will be written if set.
    let (key, value) = if let Some(result) = item {
        let item = result?;
        (Some(item.0), Some(item.1))
    } else {
        (Some(Vec::<u8>::new()), None) // Empty key will later be treated as _no more element_.
    };

    if let Some(key) = key {
        write_region!(ctx, key_ptr, &key);
    }

    if let Some(value) = value {
        write_region!(ctx, value_ptr, &value);
    }

    Ok(errors::NONE)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary, AllBalanceResponse, BankQuery, HumanAddr, Never, QueryRequest,
        SystemError, WasmQuery,
    };
    use wasmer_runtime_core::{imports, typed_func::Func, Instance as WasmerInstance};

    use crate::backends::compile;
    use crate::context::{move_into_context, set_storage_readonly, setup_context};
    #[cfg(feature = "iterator")]
    use crate::conversion::to_u32;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::traits::ReadonlyStorage;
    use crate::FfiError;

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MS = MockStorage;
    type MQ = MockQuerier;

    // prepared data
    static KEY1: &[u8] = b"ant";
    static VALUE1: &[u8] = b"insect";
    static KEY2: &[u8] = b"tree";
    static VALUE2: &[u8] = b"plant";

    // this account has some coins
    static INIT_ADDR: &str = "someone";
    static INIT_AMOUNT: u128 = 500;
    static INIT_DENOM: &str = "TOKEN";

    fn make_instance() -> Box<WasmerInstance> {
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
        let mut instance = Box::from(module.instantiate(&import_obj).unwrap());
        set_storage_readonly::<MS, MQ>(instance.context_mut(), false);
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

    fn write_data(wasmer_instance: &mut WasmerInstance, data: &[u8]) -> u32 {
        let allocate: Func<u32, u32> = wasmer_instance
            .exports
            .get("allocate")
            .expect("error getting function");
        let region_ptr = allocate
            .call(data.len() as u32)
            .expect("error calling allocate");
        write_region(wasmer_instance.context_mut(), region_ptr, data).expect("error writing");
        region_ptr
    }

    fn create_empty(wasmer_instance: &mut WasmerInstance, capacity: u32) -> u32 {
        let allocate: Func<u32, u32> = wasmer_instance
            .exports
            .get("allocate")
            .expect("error getting function");
        let region_ptr = allocate.call(capacity).expect("error calling allocate");
        region_ptr
    }

    /// A Region reader that is just good enough for the tests in this file
    fn force_read(ctx: &mut Ctx, region_ptr: u32) -> Vec<u8> {
        read_region(ctx, region_ptr, 5000).unwrap()
    }

    #[test]
    fn do_read_works() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, value_ptr), VALUE1);
    }

    #[test]
    fn do_read_works_for_non_existent_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"I do not exist in storage");
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::read::KEY_DOES_NOT_EXIST);
        assert!(force_read(ctx, value_ptr).is_empty());
    }

    #[test]
    fn do_read_fails_for_large_key() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, &vec![7u8; 300 * 1024]);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
        assert!(force_read(ctx, value_ptr).is_empty());
    }

    #[test]
    fn do_read_fails_for_small_result_region() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = create_empty(&mut instance, 3);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_read::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_WRITE_TOO_SMALL);
        assert!(force_read(ctx, value_ptr).is_empty());
    }

    #[test]
    fn do_write_works() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"new value");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);

        let val = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            Ok(store.get(b"new storage key").expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(b"new value".to_vec()));
    }

    #[test]
    fn do_write_can_override() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, KEY1);
        let value_ptr = write_data(&mut instance, VALUE2);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);

        let val = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            Ok(store.get(KEY1).expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(VALUE2.to_vec()));
    }

    #[test]
    fn do_write_works_for_empty_value() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);

        let val = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_write_fails_for_large_value() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, &vec![5u8; 300 * 1024]);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_write_is_prohibited_in_readonly_contexts() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"new value");

        let ctx = instance.context_mut();
        leave_default_data(ctx);
        set_storage_readonly::<MS, MQ>(ctx, true);

        let result = do_write::<MS, MQ>(ctx, key_ptr, value_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_remove_works() {
        let mut instance = make_instance();

        let existing_key = KEY1;
        let key_ptr = write_data(&mut instance, existing_key);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_remove::<MS, MQ>(ctx, key_ptr);
        assert_eq!(result.unwrap(), errors::NONE);

        let value = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        let result = do_remove::<MS, MQ>(ctx, key_ptr);
        // Note: right now we cannot differnetiate between an existent and a non-existent key
        assert_eq!(result.unwrap(), errors::NONE);

        let value = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        let result = do_remove::<MS, MQ>(ctx, key_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_remove_is_prohibited_in_readonly_contexts() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"a storage key");

        let ctx = instance.context_mut();
        leave_default_data(ctx);
        set_storage_readonly::<MS, MQ>(ctx, true);

        let result = do_remove::<MS, MQ>(ctx, key_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_canonicalize_address_works() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, b"foo");
        let dest_ptr = create_empty(&mut instance, 8);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_canonicalize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, dest_ptr), b"foo\0\0\0\0\0");
    }

    #[test]
    fn do_canonicalize_address_fails_for_invalid_input() {
        let mut instance = make_instance();

        let source_ptr1 = write_data(&mut instance, b"fo\x80o"); // invalid UTF-8 (fo�o)
        let source_ptr2 = write_data(&mut instance, b""); // empty
        let source_ptr3 = write_data(&mut instance, b"addressexceedingaddressspace"); // too long
        let dest_ptr = create_empty(&mut instance, 8);

        let ctx = instance.context_mut();
        leave_default_data(ctx);
        let api = MockApi::new(8);

        let result = do_canonicalize_address(api, ctx, source_ptr1, dest_ptr);
        assert_eq!(result.unwrap(), errors::canonicalize::INVALID_INPUT);

        // TODO: would be nice if do_canonicalize_address could differentiate between different errors
        // from Api.canonical_address and return INVALID_INPUT for those cases as well.
        let result = do_canonicalize_address(api, ctx, source_ptr2, dest_ptr);
        match result.unwrap_err() {
            VmError::FfiErr {
                source: FfiError::Other { .. },
            } => {}
            err => panic!("Incorrect error returned: {:?}", err),
        };

        let result = do_canonicalize_address(api, ctx, source_ptr3, dest_ptr);
        match result.unwrap_err() {
            VmError::FfiErr {
                source: FfiError::Other { .. },
            } => {}
            err => panic!("Incorrect error returned: {:?}", err),
        };
    }

    #[test]
    fn do_canonicalize_address_fails_for_large_inputs() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, &vec![61; 100]);
        let dest_ptr = create_empty(&mut instance, 8);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_canonicalize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_canonicalize_address_fails_for_small_destination_region() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, b"foo");
        let dest_ptr = create_empty(&mut instance, 7);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_canonicalize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::REGION_WRITE_TOO_SMALL);
    }

    #[test]
    fn do_humanize_address_works() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, b"foo\0\0\0\0\0");
        let dest_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_humanize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, dest_ptr), b"foo");
    }

    #[test]
    fn do_humanize_address_fails_for_invalid_canonical_length() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, b"foo\0\0");
        let dest_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_humanize_address(api, ctx, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::FfiErr {
                source: FfiError::Other { .. },
            } => {}
            err => panic!("Incorrect error returned: {:?}", err),
        };
    }

    #[test]
    fn do_humanize_address_fails_for_input_too_long() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, &vec![61; 33]);
        let dest_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_humanize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::REGION_READ_LENGTH_TOO_BIG);
    }

    #[test]
    fn do_humanize_address_fails_for_destination_region_too_small() {
        let mut instance = make_instance();

        let source_ptr = write_data(&mut instance, b"foo\0\0\0\0\0");
        let dest_ptr = create_empty(&mut instance, 2);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let api = MockApi::new(8);
        let result = do_humanize_address(api, ctx, source_ptr, dest_ptr);
        assert_eq!(result.unwrap(), errors::REGION_WRITE_TOO_SMALL);
    }

    #[test]
    fn do_query_chain_works() {
        let mut instance = make_instance();

        let request: QueryRequest<Never> = QueryRequest::Bank(BankQuery::AllBalances {
            address: HumanAddr::from(INIT_ADDR),
        });
        let request_data = cosmwasm_std::to_vec(&request).unwrap();
        let request_ptr = write_data(&mut instance, &request_data);
        let response_ptr = create_empty(&mut instance, 1000);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_query_chain::<MS, MQ>(ctx, request_ptr, response_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        let response = force_read(ctx, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        let query_result_inner = query_result.unwrap();
        let query_result_inner_inner = query_result_inner.unwrap();
        let parsed_again: AllBalanceResponse = from_binary(&query_result_inner_inner).unwrap();
        assert_eq!(parsed_again.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    fn do_query_chain_fails_for_broken_request() {
        let mut instance = make_instance();

        let request = b"Not valid JSON for sure";
        let request_ptr = write_data(&mut instance, request);
        let response_ptr = create_empty(&mut instance, 1000);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_query_chain::<MS, MQ>(ctx, request_ptr, response_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        let response = force_read(ctx, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        match query_result {
            Ok(_) => panic!("This must not succeed"),
            Err(SystemError::InvalidRequest { request: err, .. }) => {
                assert_eq!(err.as_slice(), request)
            }
            Err(error) => panic!("Unexpeted error: {:?}", error),
        }
    }

    #[test]
    fn do_query_chain_fails_for_missing_contract() {
        let mut instance = make_instance();

        let request: QueryRequest<Never> = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from("non-existent"),
            msg: Binary::from(b"{}" as &[u8]),
        });
        let request_data = cosmwasm_std::to_vec(&request).unwrap();
        let request_ptr = write_data(&mut instance, &request_data);
        let response_ptr = create_empty(&mut instance, 1000);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let result = do_query_chain::<MS, MQ>(ctx, request_ptr, response_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        let response = force_read(ctx, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        match query_result {
            Ok(_) => panic!("This must not succeed"),
            Err(SystemError::NoSuchContract { addr }) => {
                assert_eq!(addr, HumanAddr::from("non-existent"))
            }
            Err(error) => panic!("Unexpeted error: {:?}", error),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let id = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_descending_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let id = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Descending.into()).unwrap())
            .expect("ID must not be negative");
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
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

        let id = to_u32(do_scan::<MS, MQ>(ctx, start, end, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_multiple_iterators() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // unbounded, ascending and descending
        let id1 = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");
        let id2 = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Descending.into()).unwrap())
            .expect("ID must not be negative");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // first item, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        // second item, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // first item, second iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // end, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert!(item.is_none());

        // second item, second iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_works() {
        let mut instance = make_instance();

        let key_ptr = create_empty(&mut instance, 50);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");

        // Entry 1
        let result = do_next::<MS, MQ>(ctx, id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, key_ptr), KEY1);
        assert_eq!(force_read(ctx, value_ptr), VALUE1);

        // Entry 2
        let result = do_next::<MS, MQ>(ctx, id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, key_ptr), KEY2);
        assert_eq!(force_read(ctx, value_ptr), VALUE2);

        // End
        let result = do_next::<MS, MQ>(ctx, id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::NONE);
        assert_eq!(force_read(ctx, key_ptr), b"");
        // API makes no guarantees for value_ptr in this case
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_fails_for_non_existent_id() {
        let mut instance = make_instance();

        let key_ptr = create_empty(&mut instance, 50);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let non_existent_id = 42u32;
        let result = do_next::<MS, MQ>(ctx, non_existent_id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::next::ITERATOR_DOES_NOT_EXIST);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_fails_for_key_region_too_small() {
        let mut instance = make_instance();

        let key_ptr = create_empty(&mut instance, 1);
        let value_ptr = create_empty(&mut instance, 50);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");

        let result = do_next::<MS, MQ>(ctx, id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_WRITE_TOO_SMALL);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_fails_for_value_region_too_small() {
        let mut instance = make_instance();

        let key_ptr = create_empty(&mut instance, 50);
        let value_ptr = create_empty(&mut instance, 1);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = to_u32(do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap())
            .expect("ID must not be negative");

        let result = do_next::<MS, MQ>(ctx, id, key_ptr, value_ptr);
        assert_eq!(result.unwrap(), errors::REGION_WRITE_TOO_SMALL);
    }
}
