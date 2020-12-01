//! Import implementations

#[cfg(feature = "iterator")]
use std::convert::TryInto;

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr};

use crate::backend::{Api, BackendError, Querier, Storage};
use crate::context::{process_gas_info, Env};
use crate::conversion::to_u32;
use crate::errors::{CommunicationError, VmError, VmResult};
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};
use crate::serde::to_vec;
use crate::wasm_backend::get_gas_left;

/// A kibi (kilo binary)
const KI: usize = 1024;
/// A mibi (mega binary)
const MI: usize = 1024 * 1024;
/// Max key length for db_write (i.e. when VM reads from Wasm memory)
const MAX_LENGTH_DB_KEY: usize = 64 * KI;
/// Max key length for db_write (i.e. when VM reads from Wasm memory)
const MAX_LENGTH_DB_VALUE: usize = 128 * KI;
/// Typically 20 (Cosmos SDK, Ethereum) or 32 (Nano, Substrate)
const MAX_LENGTH_CANONICAL_ADDRESS: usize = 32;
/// The maximum allowed size for bech32 (https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#bech32)
const MAX_LENGTH_HUMAN_ADDRESS: usize = 90;
const MAX_LENGTH_QUERY_CHAIN_REQUEST: usize = 64 * KI;
/// Max length for a debug message
const MAX_LENGTH_DEBUG: usize = 2 * MI;

// The block of native_* prefixed functions is tailored for Wasmer's
// Function::new_native_with_env interface. Those require an env in the first
// argument and cannot capiture other variables such as the Api.

pub fn native_db_read<S: Storage, Q: Querier>(env: &Env<S, Q>, key_ptr: u32) -> VmResult<u32> {
    let ptr = do_read::<S, Q>(env, key_ptr)?;
    Ok(ptr)
}

pub fn native_db_write<S: Storage, Q: Querier>(
    env: &Env<S, Q>,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<()> {
    do_write(env, key_ptr, value_ptr)
}

pub fn native_db_remove<S: Storage, Q: Querier>(env: &Env<S, Q>, key_ptr: u32) -> VmResult<()> {
    do_remove(env, key_ptr)
}

pub fn native_query_chain<S: Storage, Q: Querier>(
    env: &Env<S, Q>,
    request_ptr: u32,
) -> VmResult<u32> {
    do_query_chain(env, request_ptr)
}

#[cfg(feature = "iterator")]
pub fn native_db_scan<S: Storage, Q: Querier>(
    env: &Env<S, Q>,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> VmResult<u32> {
    do_scan(env, start_ptr, end_ptr, order)
}

#[cfg(feature = "iterator")]
pub fn native_db_next<S: Storage, Q: Querier>(env: &Env<S, Q>, iterator_id: u32) -> VmResult<u32> {
    do_next(env, iterator_id)
}

//
// Import implementations
//

/// Reads a storage entry from the VM's storage into Wasm memory
fn do_read<S: Storage, Q: Querier>(env: &Env<S, Q>, key_ptr: u32) -> VmResult<u32> {
    let key = read_region(&env.memory(), key_ptr, MAX_LENGTH_DB_KEY)?;

    let (result, gas_info) = env.with_storage_from_context::<_, _>(|store| Ok(store.get(&key)))?;
    process_gas_info::<S, Q>(env, gas_info)?;
    let value = result?;

    let out_data = match value {
        Some(data) => data,
        None => return Ok(0),
    };
    write_to_contract::<S, Q>(env, &out_data)
}

/// Writes a storage entry from Wasm memory into the VM's storage
fn do_write<S: Storage, Q: Querier>(env: &Env<S, Q>, key_ptr: u32, value_ptr: u32) -> VmResult<()> {
    if env.is_storage_readonly() {
        return Err(VmError::write_access_denied());
    }

    let key = read_region(&env.memory(), key_ptr, MAX_LENGTH_DB_KEY)?;
    let value = read_region(&env.memory(), value_ptr, MAX_LENGTH_DB_VALUE)?;

    let (result, gas_info) =
        env.with_storage_from_context::<_, _>(|store| Ok(store.set(&key, &value)))?;
    process_gas_info::<S, Q>(env, gas_info)?;
    result?;

    Ok(())
}

fn do_remove<S: Storage, Q: Querier>(env: &Env<S, Q>, key_ptr: u32) -> VmResult<()> {
    if env.is_storage_readonly() {
        return Err(VmError::write_access_denied());
    }

    let key = read_region(&env.memory(), key_ptr, MAX_LENGTH_DB_KEY)?;

    let (result, gas_info) =
        env.with_storage_from_context::<_, _>(|store| Ok(store.remove(&key)))?;
    process_gas_info(env, gas_info)?;
    result?;

    Ok(())
}

pub fn do_canonicalize_address<A: Api, S: Storage, Q: Querier>(
    api: A,
    env: &Env<S, Q>,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<u32> {
    let source_data = read_region(&env.memory(), source_ptr, MAX_LENGTH_HUMAN_ADDRESS)?;
    if source_data.is_empty() {
        return Ok(write_to_contract::<S, Q>(env, b"Input is empty")?);
    }

    let source_string = match String::from_utf8(source_data) {
        Ok(s) => s,
        Err(_) => return Ok(write_to_contract::<S, Q>(env, b"Input is not valid UTF-8")?),
    };
    let human: HumanAddr = source_string.into();

    let (result, gas_info) = api.canonical_address(&human);
    process_gas_info::<S, Q>(env, gas_info)?;
    match result {
        Ok(canonical) => {
            write_region(&env.memory(), destination_ptr, canonical.as_slice())?;
            Ok(0)
        }
        Err(BackendError::UserErr { msg, .. }) => {
            Ok(write_to_contract::<S, Q>(env, msg.as_bytes())?)
        }
        Err(err) => Err(VmError::from(err)),
    }
}

pub fn do_humanize_address<A: Api, S: Storage, Q: Querier>(
    api: A,
    env: &Env<S, Q>,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<u32> {
    let canonical = Binary(read_region(
        &env.memory(),
        source_ptr,
        MAX_LENGTH_CANONICAL_ADDRESS,
    )?);

    let (result, gas_info) = api.human_address(&CanonicalAddr(canonical));
    process_gas_info::<S, Q>(env, gas_info)?;
    match result {
        Ok(human) => {
            write_region(&env.memory(), destination_ptr, human.as_str().as_bytes())?;
            Ok(0)
        }
        Err(BackendError::UserErr { msg, .. }) => {
            Ok(write_to_contract::<S, Q>(env, msg.as_bytes())?)
        }
        Err(err) => Err(VmError::from(err)),
    }
}

/// Prints a debug message to console.
/// This does not charge gas, so debug printing should be disabled when used in a blockchain module.
pub fn print_debug_message<S: Storage, Q: Querier>(
    env: &Env<S, Q>,
    message_ptr: u32,
) -> VmResult<()> {
    let message_data = read_region(&env.memory(), message_ptr, MAX_LENGTH_DEBUG)?;
    let msg = String::from_utf8_lossy(&message_data);
    println!("{}", msg);
    Ok(())
}

/// Creates a Region in the contract, writes the given data to it and returns the memory location
fn write_to_contract<S: Storage, Q: Querier>(env: &Env<S, Q>, input: &[u8]) -> VmResult<u32> {
    let target_ptr = env.with_func_from_context::<_, u32>("allocate", |allocate| {
        let out_size = to_u32(input.len())?;
        let result = allocate.call(&[out_size.into()])?;
        let ptr = result[0].unwrap_i32() as u32;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    })?;
    write_region(&env.memory(), target_ptr, input)?;
    Ok(target_ptr)
}

fn do_query_chain<S: Storage, Q: Querier>(env: &Env<S, Q>, request_ptr: u32) -> VmResult<u32> {
    let request = read_region(&env.memory(), request_ptr, MAX_LENGTH_QUERY_CHAIN_REQUEST)?;

    let gas_remaining = get_gas_left(env);
    let (result, gas_info) = env.with_querier_from_context::<_, _>(|querier| {
        Ok(querier.query_raw(&request, gas_remaining))
    })?;
    process_gas_info::<S, Q>(env, gas_info)?;
    let serialized = to_vec(&result?)?;
    write_to_contract::<S, Q>(env, &serialized)
}

#[cfg(feature = "iterator")]
fn do_scan<S: Storage, Q: Querier>(
    env: &Env<S, Q>,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> VmResult<u32> {
    let start = maybe_read_region(&env.memory(), start_ptr, MAX_LENGTH_DB_KEY)?;
    let end = maybe_read_region(&env.memory(), end_ptr, MAX_LENGTH_DB_KEY)?;
    let order: Order = order
        .try_into()
        .map_err(|_| CommunicationError::invalid_order(order))?;

    let (result, gas_info) = env.with_storage_from_context::<_, _>(|store| {
        Ok(store.scan(start.as_deref(), end.as_deref(), order))
    })?;
    process_gas_info::<S, Q>(env, gas_info)?;
    let iterator_id = result?;
    Ok(iterator_id)
}

#[cfg(feature = "iterator")]
fn do_next<S: Storage, Q: Querier>(env: &Env<S, Q>, iterator_id: u32) -> VmResult<u32> {
    let (result, gas_info) =
        env.with_storage_from_context::<_, _>(|store| Ok(store.next(iterator_id)))?;
    process_gas_info::<S, Q>(env, gas_info)?;

    // Empty key will later be treated as _no more element_.
    let (key, value) = result?.unwrap_or_else(|| (Vec::<u8>::new(), Vec::<u8>::new()));

    // Build value || key || keylen
    let keylen_bytes = to_u32(key.len())?.to_be_bytes();
    let mut out_data = value;
    out_data.reserve(key.len() + 4);
    out_data.extend(key);
    out_data.extend_from_slice(&keylen_bytes);

    write_to_contract::<S, Q>(env, &out_data)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
        SystemError, SystemResult, WasmQuery,
    };
    use std::ptr::NonNull;
    use wasmer::{imports, Function, FunctionType, Instance as WasmerInstance, Type, Val};

    use crate::backend::{BackendError, Storage};
    use crate::context::move_into_context;
    use crate::size::Size;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::wasm_backend::compile;

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MA = MockApi;
    type MS = MockStorage;
    type MQ = MockQuerier;

    // prepared data
    const KEY1: &[u8] = b"ant";
    const VALUE1: &[u8] = b"insect";
    const KEY2: &[u8] = b"tree";
    const VALUE2: &[u8] = b"plant";

    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const GAS_LIMIT: u64 = 5_000_000;
    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);

    fn make_instance() -> (Env<MS, MQ>, Box<WasmerInstance>) {
        let env = Env::new(GAS_LIMIT);

        let module = compile(&CONTRACT, TESTING_MEMORY_LIMIT).unwrap();
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
        env.set_storage_readonly(false);

        (env, instance)
    }

    fn leave_default_data(env: &Env<MS, MQ>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage.set(KEY1, VALUE1).0.expect("error setting");
        storage.set(KEY2, VALUE2).0.expect("error setting");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(env, storage, querier);
    }

    fn write_data(env: &Env<MS, MQ>, data: &[u8]) -> u32 {
        let region_ptr = env
            .with_func_from_context::<_, _>("allocate", |alloc_func| {
                let result = alloc_func
                    .call(&[(data.len() as u32).into()])
                    .expect("error calling allocate");
                let ptr = result[0].unwrap_i32() as u32;
                Ok(ptr)
            })
            .unwrap();
        write_region(&env.memory(), region_ptr, data).expect("error writing");
        region_ptr
    }

    fn create_empty(wasmer_instance: &mut WasmerInstance, capacity: u32) -> u32 {
        let allocate = wasmer_instance
            .exports
            .get_function("allocate")
            .expect("error getting function");
        let result = allocate
            .call(&[capacity.into()])
            .expect("error calling allocate");
        let region_ptr = result[0].unwrap_i32() as u32;
        region_ptr
    }

    /// A Region reader that is just good enough for the tests in this file
    fn force_read(env: &Env<MS, MQ>, region_ptr: u32) -> Vec<u8> {
        read_region(&env.memory(), region_ptr, 5000).unwrap()
    }

    #[test]
    fn do_read_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let key_ptr = write_data(&env, KEY1);
        let result = do_read::<MS, MQ>(&env, key_ptr);
        let value_ptr = result.unwrap();
        assert!(value_ptr > 0);
        assert_eq!(force_read(&env, value_ptr as u32), VALUE1);
    }

    #[test]
    fn do_read_works_for_non_existent_key() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let key_ptr = write_data(&env, b"I do not exist in storage");
        let result = do_read::<MS, MQ>(&env, key_ptr);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn do_read_fails_for_large_key() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        let key_ptr = write_data(&env, &vec![7u8; 300 * 1024]);
        let result = do_read::<MS, MQ>(&env, key_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
            } => assert_eq!(length, 300 * 1024),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_write_works() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, b"new storage key");
        let value_ptr = write_data(&env, b"new value");

        leave_default_data(&env);

        do_write::<MS, MQ>(&env, key_ptr, value_ptr).unwrap();

        let val = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store
                    .get(b"new storage key")
                    .0
                    .expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(b"new value".to_vec()));
    }

    #[test]
    fn do_write_can_override() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, KEY1);
        let value_ptr = write_data(&env, VALUE2);

        leave_default_data(&env);

        do_write::<MS, MQ>(&env, key_ptr, value_ptr).unwrap();

        let val = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(KEY1).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(VALUE2.to_vec()));
    }

    #[test]
    fn do_write_works_for_empty_value() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, b"new storage key");
        let value_ptr = write_data(&env, b"");

        leave_default_data(&env);

        do_write::<MS, MQ>(&env, key_ptr, value_ptr).unwrap();

        let val = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store
                    .get(b"new storage key")
                    .0
                    .expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(b"".to_vec()));
    }

    #[test]
    fn do_write_fails_for_large_key() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, &vec![4u8; 300 * 1024]);
        let value_ptr = write_data(&env, b"new value");

        leave_default_data(&env);

        let result = do_write::<MS, MQ>(&env, key_ptr, value_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_KEY);
            }
            err => panic!("unexpected error: {:?}", err),
        };
    }

    #[test]
    fn do_write_fails_for_large_value() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, b"new storage key");
        let value_ptr = write_data(&env, &vec![5u8; 300 * 1024]);

        leave_default_data(&env);

        let result = do_write::<MS, MQ>(&env, key_ptr, value_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_VALUE);
            }
            err => panic!("unexpected error: {:?}", err),
        };
    }

    #[test]
    fn do_write_is_prohibited_in_readonly_contexts() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, b"new storage key");
        let value_ptr = write_data(&env, b"new value");

        leave_default_data(&env);
        env.set_storage_readonly(true);

        let result = do_write::<MS, MQ>(&env, key_ptr, value_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_remove_works() {
        let (env, _instance) = make_instance();

        let existing_key = KEY1;
        let key_ptr = write_data(&env, existing_key);

        leave_default_data(&env);

        env.with_storage_from_context::<_, _>(|store| {
            println!("{:?}", store);
            Ok(())
        })
        .unwrap();

        do_remove::<MS, MQ>(&env, key_ptr).unwrap();

        env.with_storage_from_context::<_, _>(|store| {
            println!("{:?}", store);
            Ok(())
        })
        .unwrap();

        let value = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(existing_key).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_remove_works_for_non_existent_key() {
        let (env, _instance) = make_instance();

        let non_existent_key = b"I do not exist";
        let key_ptr = write_data(&env, non_existent_key);

        leave_default_data(&env);

        // Note: right now we cannot differnetiate between an existent and a non-existent key
        do_remove::<MS, MQ>(&env, key_ptr).unwrap();

        let value = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(non_existent_key).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_remove_fails_for_large_key() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, &vec![26u8; 300 * 1024]);

        leave_default_data(&env);

        let result = do_remove::<MS, MQ>(&env, key_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_KEY);
            }
            err => panic!("unexpected error: {:?}", err),
        };
    }

    #[test]
    fn do_remove_is_prohibited_in_readonly_contexts() {
        let (env, _instance) = make_instance();

        let key_ptr = write_data(&env, b"a storage key");

        leave_default_data(&env);
        env.set_storage_readonly(true);

        let result = do_remove::<MS, MQ>(&env, key_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn do_canonicalize_address_works() {
        let (env, mut instance) = make_instance();
        let api = MockApi::default();

        let source_ptr = write_data(&env, b"foo");
        let dest_ptr = create_empty(&mut instance, api.canonical_length as u32);

        leave_default_data(&env);

        let api = MockApi::default();
        do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr).unwrap();
        let data = force_read(&env, dest_ptr);
        assert_eq!(data.len(), api.canonical_length);
    }

    #[test]
    fn do_canonicalize_address_reports_invalid_input_back_to_contract() {
        let (env, mut instance) = make_instance();

        let source_ptr1 = write_data(&env, b"fo\x80o"); // invalid UTF-8 (fo�o)
        let source_ptr2 = write_data(&env, b""); // empty
        let source_ptr3 = write_data(&env, b"addressexceedingaddressspace"); // too long
        let dest_ptr = create_empty(&mut instance, 8);

        leave_default_data(&env);
        let api = MockApi::default();

        let res = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr1, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&env, res)).unwrap();
        assert_eq!(err, "Input is not valid UTF-8");

        let res = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr2, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&env, res)).unwrap();
        assert_eq!(err, "Input is empty");

        let res = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr3, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&env, res)).unwrap();
        assert_eq!(err, "Invalid input: human address too long");
    }

    #[test]
    fn do_canonicalize_address_fails_for_broken_backend() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, b"foo");
        let dest_ptr = create_empty(&mut instance, 7);

        leave_default_data(&env);

        let api = MockApi::new_failing("Temporarily unavailable");
        let result = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::Unknown { msg, .. },
            } => {
                assert_eq!(msg.unwrap(), "Temporarily unavailable");
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
    }

    #[test]
    fn do_canonicalize_address_fails_for_large_inputs() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, &vec![61; 100]);
        let dest_ptr = create_empty(&mut instance, 8);

        leave_default_data(&env);

        let api = MockApi::default();
        let result = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 100);
                assert_eq!(max_length, 90);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
    }

    #[test]
    fn do_canonicalize_address_fails_for_small_destination_region() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, b"foo");
        let dest_ptr = create_empty(&mut instance, 7);

        leave_default_data(&env);

        let api = MockApi::default();
        let result = do_canonicalize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionTooSmall { size, required, .. },
            } => {
                assert_eq!(size, 7);
                assert_eq!(required, api.canonical_length);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
    }

    #[test]
    fn do_humanize_address_works() {
        let (env, mut instance) = make_instance();
        let api = MockApi::default();

        let source_data = vec![0x22; api.canonical_length];
        let source_ptr = write_data(&env, &source_data);
        let dest_ptr = create_empty(&mut instance, 50);

        leave_default_data(&env);

        let api = MockApi::default();
        let error_ptr = do_humanize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr).unwrap();
        assert_eq!(error_ptr, 0);
        assert_eq!(force_read(&env, dest_ptr), source_data);
    }

    #[test]
    fn do_humanize_address_reports_invalid_input_back_to_contract() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, b"foo"); // too short
        let dest_ptr = create_empty(&mut instance, 50);

        leave_default_data(&env);

        let api = MockApi::default();
        let res = do_humanize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&env, res)).unwrap();
        assert_eq!(err, "Invalid input: canonical address length not correct");
    }

    #[test]
    fn do_humanize_address_fails_for_broken_backend() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, b"foo\0\0\0\0\0");
        let dest_ptr = create_empty(&mut instance, 50);

        leave_default_data(&env);

        let api = MockApi::new_failing("Temporarily unavailable");
        let result = do_humanize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::Unknown { msg, .. },
            } => assert_eq!(msg.unwrap(), "Temporarily unavailable"),
            err => panic!("Incorrect error returned: {:?}", err),
        };
    }

    #[test]
    fn do_humanize_address_fails_for_input_too_long() {
        let (env, mut instance) = make_instance();

        let source_ptr = write_data(&env, &vec![61; 33]);
        let dest_ptr = create_empty(&mut instance, 50);

        leave_default_data(&env);

        let api = MockApi::default();
        let result = do_humanize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
            } => {
                assert_eq!(length, 33);
                assert_eq!(max_length, 32);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
    }

    #[test]
    fn do_humanize_address_fails_for_destination_region_too_small() {
        let (env, mut instance) = make_instance();
        let api = MockApi::default();

        let source_data = vec![0x22; api.canonical_length];
        let source_ptr = write_data(&env, &source_data);
        let dest_ptr = create_empty(&mut instance, 2);

        leave_default_data(&env);

        let api = MockApi::default();
        let result = do_humanize_address::<MA, MS, MQ>(api, &env, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionTooSmall { size, required, .. },
            } => {
                assert_eq!(size, 2);
                assert_eq!(required, api.canonical_length);
            }
            err => panic!("Incorrect error returned: {:?}", err),
        }
    }

    #[test]
    fn do_query_chain_works() {
        let (env, _instance) = make_instance();

        let request: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::AllBalances {
            address: HumanAddr::from(INIT_ADDR),
        });
        let request_data = cosmwasm_std::to_vec(&request).unwrap();
        let request_ptr = write_data(&env, &request_data);

        leave_default_data(&env);

        let response_ptr = do_query_chain::<MS, MQ>(&env, request_ptr).unwrap();
        let response = force_read(&env, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        let query_result_inner = query_result.unwrap();
        let query_result_inner_inner = query_result_inner.unwrap();
        let parsed_again: AllBalanceResponse = from_binary(&query_result_inner_inner).unwrap();
        assert_eq!(parsed_again.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    fn do_query_chain_fails_for_broken_request() {
        let (env, _instance) = make_instance();

        let request = b"Not valid JSON for sure";
        let request_ptr = write_data(&env, request);

        leave_default_data(&env);

        let response_ptr = do_query_chain::<MS, MQ>(&env, request_ptr).unwrap();
        let response = force_read(&env, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        match query_result {
            SystemResult::Ok(_) => panic!("This must not succeed"),
            SystemResult::Err(SystemError::InvalidRequest { request: err, .. }) => {
                assert_eq!(err.as_slice(), request)
            }
            SystemResult::Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn do_query_chain_fails_for_missing_contract() {
        let (env, _instance) = make_instance();

        let request: QueryRequest<Empty> = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from("non-existent"),
            msg: Binary::from(b"{}" as &[u8]),
        });
        let request_data = cosmwasm_std::to_vec(&request).unwrap();
        let request_ptr = write_data(&env, &request_data);

        leave_default_data(&env);

        let response_ptr = do_query_chain::<MS, MQ>(&env, request_ptr).unwrap();
        let response = force_read(&env, response_ptr);

        let query_result: cosmwasm_std::QuerierResult =
            cosmwasm_std::from_slice(&response).unwrap();
        match query_result {
            SystemResult::Ok(_) => panic!("This must not succeed"),
            SystemResult::Err(SystemError::NoSuchContract { addr }) => {
                assert_eq!(addr, HumanAddr::from("non-existent"))
            }
            SystemResult::Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // set up iterator over all space
        let id = do_scan::<MS, MQ>(&env, 0, 0, Order::Ascending.into()).unwrap();
        assert_eq!(1, id);

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_unbound_descending_works() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // set up iterator over all space
        let id = do_scan::<MS, MQ>(&env, 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(1, id);

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_bound_works() {
        let (env, _instance) = make_instance();

        let start = write_data(&env, b"anna");
        let end = write_data(&env, b"bert");

        leave_default_data(&env);

        let id = do_scan::<MS, MQ>(&env, start, end, Order::Ascending.into()).unwrap();

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_multiple_iterators() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // unbounded, ascending and descending
        let id1 = do_scan::<MS, MQ>(&env, 0, 0, Order::Ascending.into()).unwrap();
        let id2 = do_scan::<MS, MQ>(&env, 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // first item, first iterator
        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        // second item, first iterator
        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // first item, second iterator
        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id2)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // end, first iterator
        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert!(item.0.unwrap().is_none());

        // second item, second iterator
        let item = env
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id2)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_scan_errors_for_invalid_order_value() {
        let (env, _instance) = make_instance();
        leave_default_data(&env);

        // set up iterator over all space
        let result = do_scan::<MS, MQ>(&env, 0, 0, 42);
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
        let (env, _instance) = make_instance();

        leave_default_data(&env);

        let id = do_scan::<MS, MQ>(&env, 0, 0, Order::Ascending.into()).unwrap();

        // Entry 1
        let kv_region_ptr = do_next::<MS, MQ>(&env, id).unwrap();
        assert_eq!(
            force_read(&env, kv_region_ptr),
            [VALUE1, KEY1, b"\0\0\0\x03"].concat()
        );

        // Entry 2
        let kv_region_ptr = do_next::<MS, MQ>(&env, id).unwrap();
        assert_eq!(
            force_read(&env, kv_region_ptr),
            [VALUE2, KEY2, b"\0\0\0\x04"].concat()
        );

        // End
        let kv_region_ptr = do_next::<MS, MQ>(&env, id).unwrap();
        assert_eq!(force_read(&env, kv_region_ptr), b"\0\0\0\0");
        // API makes no guarantees for value_ptr in this case
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_next_fails_for_non_existent_id() {
        let (env, _instance) = make_instance();

        leave_default_data(&env);

        let non_existent_id = 42u32;
        let result = do_next::<MS, MQ>(&env, non_existent_id);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::IteratorDoesNotExist { id, .. },
            } => assert_eq!(id, non_existent_id),
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
