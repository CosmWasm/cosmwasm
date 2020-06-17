//! Import implementations

#[cfg(feature = "iterator")]
use std::convert::TryInto;

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr};
use wasmer_runtime_core::vm::Ctx;

#[cfg(feature = "iterator")]
use crate::context::{add_iterator, with_iterator_from_context};
use crate::context::{
    is_storage_readonly, try_consume_gas, with_func_from_context, with_querier_from_context,
    with_storage_from_context,
};
use crate::conversion::to_u32;
use crate::errors::{CommunicationError, VmError, VmResult};
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};
use crate::serde::to_vec;
use crate::traits::{Api, Querier, Storage};

/// A kibi (kilo binary)
static KI: usize = 1024;
/// Max key length for db_write (i.e. when VM reads from Wasm memory)
static MAX_LENGTH_DB_KEY: usize = 64 * KI;
/// Max key length for db_write (i.e. when VM reads from Wasm memory)
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

    // unused block (-1_000_2xx)
    // unused block (-1_000_3xx)
    // unused block (-1_000_4xx)

    /// db_read errors (-1_001_0xx)
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

    // query_chain errors (-1_003_0xx)

    // The -2_xxx_xxx namespace is reserved for #[cfg(feature = "iterator")]
    // db_scan errors (-2_000_0xx)
    // db_next errors (-2_000_1xx)
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
pub fn do_read<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32) -> VmResult<u32> {
    let key = read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY)?;
    // `Ok(expr?)` used to convert the error variant.
    let (value, used_gas): (Option<Vec<u8>>, u64) =
        with_storage_from_context::<S, Q, _, _>(ctx, |store| Ok(store.get(&key)?))?;
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    let out_data = match value {
        Some(data) => data,
        None => return Ok(0),
    };

    let out_ptr = with_func_from_context::<S, Q, u32, u32, _, _>(ctx, "allocate", |allocate| {
        let out_size = to_u32(out_data.len())?;
        let ptr = allocate.call(out_size)?;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    })?;
    write_region(ctx, out_ptr, &out_data)?;
    Ok(out_ptr)
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_write<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<()> {
    if is_storage_readonly::<S, Q>(ctx) {
        return Err(VmError::write_access_denied());
    }

    let key = read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY)?;
    let value = read_region(ctx, value_ptr, MAX_LENGTH_DB_VALUE)?;
    let used_gas =
        with_storage_from_context::<S, Q, _, _>(ctx, |store| Ok(store.set(&key, &value)?))?;
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    Ok(())
}

pub fn do_remove<S: Storage, Q: Querier>(ctx: &mut Ctx, key_ptr: u32) -> VmResult<()> {
    if is_storage_readonly::<S, Q>(ctx) {
        return Err(VmError::write_access_denied());
    }

    let key = read_region(ctx, key_ptr, MAX_LENGTH_DB_KEY)?;
    let used_gas = with_storage_from_context::<S, Q, _, _>(ctx, |store| Ok(store.remove(&key)?))?;
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    Ok(())
}

pub fn do_canonicalize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<i32> {
    let human_data = read_region(ctx, source_ptr, MAX_LENGTH_HUMAN_ADDRESS)?;
    let human = match String::from_utf8(human_data) {
        Ok(human_str) => HumanAddr(human_str),
        Err(_) => return Ok(errors::canonicalize::INVALID_INPUT),
    };
    let canon = api.canonical_address(&human)?;
    Ok(write_region!(ctx, destination_ptr, canon.as_slice()))
}

pub fn do_humanize_address<A: Api>(
    api: A,
    ctx: &mut Ctx,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<i32> {
    let canonical = Binary(read_region(ctx, source_ptr, MAX_LENGTH_CANONICAL_ADDRESS)?);
    let human = api.human_address(&CanonicalAddr(canonical))?;
    Ok(write_region!(
        ctx,
        destination_ptr,
        human.as_str().as_bytes()
    ))
}

pub fn do_query_chain<S: Storage, Q: Querier>(ctx: &mut Ctx, request_ptr: u32) -> VmResult<u32> {
    let request = read_region(ctx, request_ptr, MAX_LENGTH_QUERY_CHAIN_REQUEST)?;

    let (res, used_gas) =
        with_querier_from_context::<S, Q, _, _>(ctx, |querier| Ok(querier.raw_query(&request)?))?;
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    let serialized = to_vec(&res)?;
    let out_ptr = with_func_from_context::<S, Q, u32, u32, _, _>(ctx, "allocate", |allocate| {
        let out_size = to_u32(serialized.len())?;
        let ptr = allocate.call(out_size)?;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    })?;
    write_region(ctx, out_ptr, &serialized)?;
    Ok(out_ptr)
}

#[cfg(feature = "iterator")]
pub fn do_scan<S: Storage + 'static, Q: Querier>(
    ctx: &mut Ctx,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> VmResult<u32> {
    let start = maybe_read_region(ctx, start_ptr, MAX_LENGTH_DB_KEY)?;
    let end = maybe_read_region(ctx, end_ptr, MAX_LENGTH_DB_KEY)?;
    let order: Order = order
        .try_into()
        .map_err(|_| CommunicationError::invalid_order(order))?;
    let (iterator, used_gas) = with_storage_from_context::<S, Q, _, _>(ctx, |store| {
        Ok(store.range(start.as_deref(), end.as_deref(), order)?)
    })?;
    // Gas is consumed for creating an iterator if the first key in the DB has a value
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    let new_id = add_iterator::<S, Q>(ctx, iterator);
    Ok(new_id)
}

#[cfg(feature = "iterator")]
pub fn do_next<S: Storage, Q: Querier>(ctx: &mut Ctx, iterator_id: u32) -> VmResult<u32> {
    let item = with_iterator_from_context::<S, Q, _, _>(ctx, iterator_id, |iter| Ok(iter.next()))?;

    let (kv, used_gas) = item?;
    try_consume_gas::<S, Q>(ctx, used_gas)?;

    // Empty key will later be treated as _no more element_.
    let (key, value) = kv.unwrap_or_else(|| (Vec::<u8>::new(), Vec::<u8>::new()));

    // Build value || key || keylen
    let keylen_bytes = to_u32(key.len())?.to_be_bytes();
    let mut out_data = value;
    out_data.reserve(key.len() + 4);
    out_data.extend(key);
    out_data.extend_from_slice(&keylen_bytes);

    let out_ptr = with_func_from_context::<S, Q, u32, u32, _, _>(ctx, "allocate", |allocate| {
        let out_size = to_u32(out_data.len())?;
        let ptr = allocate.call(out_size)?;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    })?;
    write_region(ctx, out_ptr, &out_data)?;
    Ok(out_ptr)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary, AllBalanceResponse, BankQuery, HumanAddr, Never, QueryRequest,
        SystemError, WasmQuery,
    };
    use std::ptr::NonNull;
    use wasmer_runtime_core::{imports, typed_func::Func, Instance as WasmerInstance};

    use crate::backends::compile;
    use crate::context::{
        move_into_context, set_storage_readonly, set_wasmer_instance, setup_context,
    };
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
                "db_read" => Func::new(|_a: u32| -> u32 { 0 }),
                "db_write" => Func::new(|_a: u32, _b: u32| {}),
                "db_remove" => Func::new(|_a: u32| {}),
                "db_scan" => Func::new(|_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Func::new(|_a: u32| -> u32 { 0 }),
                "query_chain" => Func::new(|_a: u32| -> u32 { 0 }),
                "canonicalize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
                "humanize_address" => Func::new(|_a: i32, _b: i32| -> i32 { 0 }),
            },
        };
        let mut instance = Box::from(module.instantiate(&import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        set_wasmer_instance::<MS, MQ>(instance.context_mut(), Some(instance_ptr));
        set_storage_readonly::<MS, MQ>(instance.context_mut(), false);

        instance
    }

    fn leave_default_data(ctx: &mut Ctx) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage.set(KEY1, VALUE1).expect("error setting");
        storage.set(KEY2, VALUE2).expect("error setting");
        let querier: MockQuerier<Never> =
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
        leave_default_data(instance.context_mut());

        let key_ptr = write_data(&mut instance, KEY1);
        let ctx = instance.context_mut();
        let result = do_read::<MS, MQ>(ctx, key_ptr);
        let value_ptr = result.unwrap();
        assert!(value_ptr > 0);
        assert_eq!(force_read(ctx, value_ptr as u32), VALUE1);
    }

    #[test]
    fn do_read_works_for_non_existent_key() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        let key_ptr = write_data(&mut instance, b"I do not exist in storage");
        let ctx = instance.context_mut();
        let result = do_read::<MS, MQ>(ctx, key_ptr);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn do_read_fails_for_large_key() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        let key_ptr = write_data(&mut instance, &vec![7u8; 300 * 1024]);
        let ctx = instance.context_mut();
        let result = do_read::<MS, MQ>(ctx, key_ptr);
        match result.unwrap_err() {
            VmError::RegionLengthTooBig { length, .. } => assert_eq!(length, 300 * 1024),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_write_works() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, b"new value");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        do_write::<MS, MQ>(ctx, key_ptr, value_ptr).unwrap();

        let (val, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        do_write::<MS, MQ>(ctx, key_ptr, value_ptr).unwrap();

        let (val, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        do_write::<MS, MQ>(ctx, key_ptr, value_ptr).unwrap();

        let (val, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        match do_write::<MS, MQ>(ctx, key_ptr, value_ptr).unwrap_err() {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_KEY);
            }
            err => panic!("unexpected error: {:?}", err),
        };
    }

    #[test]
    fn do_write_fails_for_large_value() {
        let mut instance = make_instance();

        let key_ptr = write_data(&mut instance, b"new storage key");
        let value_ptr = write_data(&mut instance, &vec![5u8; 300 * 1024]);

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        match do_write::<MS, MQ>(ctx, key_ptr, value_ptr).unwrap_err() {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_VALUE);
            }
            err => panic!("unexpected error: {:?}", err),
        };
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

        do_remove::<MS, MQ>(ctx, key_ptr).unwrap();

        let (value, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        // Note: right now we cannot differnetiate between an existent and a non-existent key
        do_remove::<MS, MQ>(ctx, key_ptr).unwrap();

        let (value, _used_gas) = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
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

        match do_remove::<MS, MQ>(ctx, key_ptr).unwrap_err() {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_KEY);
            }
            err => panic!("unexpected error: {:?}", err),
        };
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
        match result.unwrap_err() {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 100);
                assert_eq!(max_length, 90);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
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
        match result.unwrap_err() {
            VmError::RegionLengthTooBig {
                length, max_length, ..
            } => {
                assert_eq!(length, 33);
                assert_eq!(max_length, 32);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
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

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let response_ptr = do_query_chain::<MS, MQ>(ctx, request_ptr).unwrap();
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

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let response_ptr = do_query_chain::<MS, MQ>(ctx, request_ptr).unwrap();
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

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let response_ptr = do_query_chain::<MS, MQ>(ctx, request_ptr).unwrap();
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
        let id = do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap();
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.unwrap().0.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_descending_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let id = do_scan::<MS, MQ>(ctx, 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(1, id);

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.unwrap().0.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_bound_works() {
        let mut instance = make_instance();

        let start = write_data(&mut instance, b"anna");
        let end = write_data(&mut instance, b"bert");

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = do_scan::<MS, MQ>(ctx, start, end, Order::Ascending.into()).unwrap();

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id, |iter| Ok(iter.next())).unwrap();
        assert!(item.unwrap().0.is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_multiple_iterators() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // unbounded, ascending and descending
        let id1 = do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap();
        let id2 = do_scan::<MS, MQ>(ctx, 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // first item, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        // second item, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // first item, second iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // end, first iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id1, |iter| Ok(iter.next())).unwrap();
        assert!(item.unwrap().0.is_none());

        // second item, second iterator
        let item =
            with_iterator_from_context::<MS, MQ, _, _>(ctx, id2, |iter| Ok(iter.next())).unwrap();
        assert_eq!(item.unwrap().0.unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_errors_for_invalid_order_value() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // set up iterator over all space
        let result = do_scan::<MS, MQ>(ctx, 0, 0, 42);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::InvalidOrder { .. },
            } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_works() {
        let mut instance = make_instance();

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let id = do_scan::<MS, MQ>(ctx, 0, 0, Order::Ascending.into()).unwrap();

        // Entry 1
        let kv_region_ptr = do_next::<MS, MQ>(ctx, id).unwrap();
        assert_eq!(
            force_read(ctx, kv_region_ptr),
            [VALUE1, KEY1, b"\0\0\0\x03"].concat()
        );

        // Entry 2
        let kv_region_ptr = do_next::<MS, MQ>(ctx, id).unwrap();
        assert_eq!(
            force_read(ctx, kv_region_ptr),
            [VALUE2, KEY2, b"\0\0\0\x04"].concat()
        );

        // End
        let kv_region_ptr = do_next::<MS, MQ>(ctx, id).unwrap();
        assert_eq!(force_read(ctx, kv_region_ptr), b"\0\0\0\0");
        // API makes no guarantees for value_ptr in this case
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_fails_for_non_existent_id() {
        let mut instance = make_instance();

        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let non_existent_id = 42u32;
        let result = do_next::<MS, MQ>(ctx, non_existent_id);
        match result.unwrap_err() {
            VmError::IteratorDoesNotExist { id, .. } => assert_eq!(id, non_existent_id),
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
