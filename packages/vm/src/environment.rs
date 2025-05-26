//! Internal details to be used by instance.rs only
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use derive_more::Debug;
use wasmer::{AsStoreMut, Instance as WasmerInstance, Memory, MemoryView, Value};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

use crate::backend::{BackendApi, GasInfo, Querier, Storage};
use crate::errors::{VmError, VmResult};

/// Keep this as low as necessary to avoid deepy nested errors like this:
///
/// ```plain
/// RuntimeErr { msg: "Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Maximum call depth exceeded." }
/// ```
const MAX_CALL_DEPTH: usize = 2;

/// Never can never be instantiated.
/// Replace this with the [never primitive type](https://doc.rust-lang.org/std/primitive.never.html) when stable.
#[derive(Debug)]
pub enum Never {}

/** gas config data */

#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct GasConfig {
    /// Gas costs of VM (not Backend) provided functionality
    /// secp256k1 signature verification cost
    pub secp256k1_verify_cost: u64,
    /// secp256k1 public key recovery cost
    pub secp256k1_recover_pubkey_cost: u64,
    /// secp256r1 signature verification cost
    pub secp256r1_verify_cost: u64,
    /// secp256r1 public key recovery cost
    pub secp256r1_recover_pubkey_cost: u64,
    /// ed25519 signature verification cost
    pub ed25519_verify_cost: u64,
    /// ed25519 batch signature verification cost
    pub ed25519_batch_verify_cost: LinearGasCost,
    /// ed25519 batch signature verification cost (single public key)
    pub ed25519_batch_verify_one_pubkey_cost: LinearGasCost,
    /// bls12-381 aggregate cost (g1)
    pub bls12_381_aggregate_g1_cost: LinearGasCost,
    /// bls12-381 aggregate cost (g2)
    pub bls12_381_aggregate_g2_cost: LinearGasCost,
    /// bls12-381 hash to g1 cost
    pub bls12_381_hash_to_g1_cost: u64,
    /// bls12-381 hash to g2 cost
    pub bls12_381_hash_to_g2_cost: u64,
    /// bls12-381 pairing equality check cost
    pub bls12_381_pairing_equality_cost: LinearGasCost,
    /// cost for writing memory regions
    pub write_region_cost: LinearGasCost,
    /// cost for reading memory regions <= 8MB
    pub read_region_small_cost: LinearGasCost,
    /// cost for reading memory regions > 8MB
    pub read_region_large_cost: LinearGasCost,
    /// cost for validating bytes into a String
    pub string_from_bytes_cost: LinearGasCost,
    /// cost for calling a host function
    pub host_call_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        // Target is 10^12 per second (see GAS.md), i.e. 10^6 gas per µ second.
        const GAS_PER_US: u64 = 1_000_000;
        Self {
            // ~96 us in crypto benchmarks
            secp256k1_verify_cost: 96 * GAS_PER_US,
            // ~194 us in crypto benchmarks
            secp256k1_recover_pubkey_cost: 194 * GAS_PER_US,
            // ~279 us in crypto benchmarks
            secp256r1_verify_cost: 279 * GAS_PER_US,
            // ~592 us in crypto benchmarks
            secp256r1_recover_pubkey_cost: 592 * GAS_PER_US,
            // ~35 us in crypto benchmarks
            ed25519_verify_cost: 35 * GAS_PER_US,
            // Calculated based on the benchmark results for `ed25519_batch_verify_{x}`.
            ed25519_batch_verify_cost: LinearGasCost {
                base: 24 * GAS_PER_US,
                per_item: 21 * GAS_PER_US,
            },
            // Calculated based on the benchmark results for `ed25519_batch_verify_one_pubkey_{x}`.
            ed25519_batch_verify_one_pubkey_cost: LinearGasCost {
                base: 36 * GAS_PER_US,
                per_item: 10 * GAS_PER_US,
            },
            // just assume the production machines have more than 4 cores, so we can half that
            bls12_381_aggregate_g1_cost: LinearGasCost {
                base: 136 * GAS_PER_US / 2,
                per_item: 24 * GAS_PER_US / 2,
            },
            bls12_381_aggregate_g2_cost: LinearGasCost {
                base: 207 * GAS_PER_US / 2,
                per_item: 49 * GAS_PER_US / 2,
            },
            bls12_381_hash_to_g1_cost: 563 * GAS_PER_US,
            bls12_381_hash_to_g2_cost: 871 * GAS_PER_US,
            bls12_381_pairing_equality_cost: LinearGasCost {
                base: 2112 * GAS_PER_US,
                per_item: 163 * GAS_PER_US,
            },
            write_region_cost: LinearGasCost {
                base: 230000,
                per_item: 570,
            },
            read_region_small_cost: LinearGasCost {
                base: 200000,
                per_item: 115,
            },
            read_region_large_cost: LinearGasCost {
                base: 0,
                per_item: 520,
            },
            string_from_bytes_cost: LinearGasCost {
                base: 28700,
                per_item: 1400,
            },
            host_call_cost: 18000,
        }
    }
}

impl GasConfig {
    pub fn read_region_cost(&self, bytes: usize) -> VmResult<u64> {
        const THRESHOLD: usize = 8 * 1000 * 1000;
        if bytes <= THRESHOLD {
            self.read_region_small_cost.total_cost(bytes as u64)
        } else {
            self.read_region_large_cost.total_cost(bytes as u64)
        }
    }
}

/// Linear gas cost model where the cost is linear in the number of items.
///
/// To calculate it, you sample the cost for a few different amounts of items and fit a line to it.
/// Let `b` be that line of best fit. Then `base = b(0)` is the y-intercept and
/// `per_item = b(1) - b(0)` the slope.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LinearGasCost {
    /// This is a flat part of the cost, charged once per batch.
    base: u64,
    /// This is the cost per item in the batch.
    per_item: u64,
}

impl LinearGasCost {
    pub fn total_cost(&self, items: u64) -> VmResult<u64> {
        self.total_cost_opt(items)
            .ok_or_else(VmError::gas_depletion)
    }

    fn total_cost_opt(&self, items: u64) -> Option<u64> {
        self.base.checked_add(self.per_item.checked_mul(items)?)
    }
}

/** context data **/

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation, including internally and externally used gas.
    /// This is set when the Environment is created and never mutated.
    ///
    /// Measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    /// Tracking the gas used in the Cosmos SDK, in CosmWasm gas units.
    pub externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }
}

/// Additional environmental information in a debug call.
///
/// The currently unused lifetime parameter 'a allows accessing referenced data in the debug implementation
/// without cloning it.
#[derive(Debug)]
#[non_exhaustive]
pub struct DebugInfo<'a> {
    pub gas_remaining: u64,
    // This field is just to allow us to add the unused lifetime parameter. It can be removed
    // at any time.
    #[doc(hidden)]
    #[debug(skip)]
    pub(crate) __lifetime: PhantomData<&'a ()>,
}

// Unfortunately we cannot create an alias for the trait (https://github.com/rust-lang/rust/issues/41517).
// So we need to copy it in a few places.
//
//                            /- BEGIN TRAIT                          END TRAIT \
//                            |                                                 |
//                            v                                                 v
pub type DebugHandlerFn = dyn for<'a, 'b> FnMut(/* msg */ &'a str, DebugInfo<'b>);

/// A environment that provides access to the ContextData.
/// The environment is cloneable but clones access the same underlying data.
pub struct Environment<A, S, Q> {
    pub memory: Option<Memory>,
    pub api: A,
    pub gas_config: GasConfig,
    data: Arc<RwLock<ContextData<S, Q>>>,
}

unsafe impl<A: BackendApi, S: Storage, Q: Querier> Send for Environment<A, S, Q> {}

unsafe impl<A: BackendApi, S: Storage, Q: Querier> Sync for Environment<A, S, Q> {}

impl<A: BackendApi, S: Storage, Q: Querier> Clone for Environment<A, S, Q> {
    fn clone(&self) -> Self {
        Environment {
            memory: None,
            api: self.api.clone(),
            gas_config: self.gas_config.clone(),
            data: self.data.clone(),
        }
    }
}

impl<A: BackendApi, S: Storage, Q: Querier> Environment<A, S, Q> {
    pub fn new(api: A, gas_limit: u64) -> Self {
        Environment {
            memory: None,
            api,
            gas_config: GasConfig::default(),
            data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
        }
    }

    pub fn set_debug_handler(&self, debug_handler: Option<Rc<RefCell<DebugHandlerFn>>>) {
        self.with_context_data_mut(|context_data| {
            context_data.debug_handler = debug_handler;
        })
    }

    pub fn debug_handler(&self) -> Option<Rc<RefCell<DebugHandlerFn>>> {
        self.with_context_data(|context_data| {
            // This clone here requires us to wrap the function in Rc instead of Box
            context_data.debug_handler.clone()
        })
    }

    fn with_context_data_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut ContextData<S, Q>) -> R,
    {
        let mut guard = self.data.as_ref().write().unwrap();
        let context_data = guard.borrow_mut();
        callback(context_data)
    }

    fn with_context_data<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&ContextData<S, Q>) -> R,
    {
        let guard = self.data.as_ref().read().unwrap();
        callback(&guard)
    }

    pub fn with_gas_state<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&GasState) -> R,
    {
        self.with_context_data(|context_data| callback(&context_data.gas_state))
    }

    pub fn with_gas_state_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut GasState) -> R,
    {
        self.with_context_data_mut(|context_data| callback(&mut context_data.gas_state))
    }

    pub fn with_wasmer_instance<C, R>(&self, callback: C) -> VmResult<R>
    where
        C: FnOnce(&WasmerInstance) -> VmResult<R>,
    {
        self.with_context_data(|context_data| match context_data.wasmer_instance {
            Some(instance_ptr) => {
                let instance_ref = unsafe { instance_ptr.as_ref() };
                callback(instance_ref)
            }
            None => Err(VmError::uninitialized_context_data("wasmer_instance")),
        })
    }

    /// Calls a function with the given name and arguments.
    /// The number of return values is variable and controlled by the guest.
    /// Usually we expect 0 or 1 return values. Use [`Self::call_function0`]
    /// or [`Self::call_function1`] to ensure the number of return values is checked.
    fn call_function(
        &self,
        store: &mut impl AsStoreMut,
        name: &str,
        args: &[Value],
    ) -> VmResult<Box<[Value]>> {
        // Clone function before calling it to avoid dead locks
        let func = self.with_wasmer_instance(|instance| {
            let func = instance.exports.get_function(name)?;
            Ok(func.clone())
        })?;
        let function_arity = func.param_arity(store);
        if args.len() != function_arity {
            return Err(VmError::function_arity_mismatch(function_arity));
        };
        self.increment_call_depth()?;
        let res = func.call(store, args).map_err(|runtime_err| -> VmError {
            self.with_wasmer_instance::<_, Never>(|instance| {
                let err: VmError = match get_remaining_points(store, instance) {
                    MeteringPoints::Remaining(_) => VmError::from(runtime_err),
                    MeteringPoints::Exhausted => VmError::gas_depletion(),
                };
                Err(err)
            })
            .unwrap_err() // with_wasmer_instance can only succeed if the callback succeeds
        });
        self.decrement_call_depth();
        res
    }

    pub fn call_function0(
        &self,
        store: &mut impl AsStoreMut,
        name: &str,
        args: &[Value],
    ) -> VmResult<()> {
        let result = self.call_function(store, name, args)?;
        let expected = 0;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(())
    }

    pub fn call_function1(
        &self,
        store: &mut impl AsStoreMut,
        name: &str,
        args: &[Value],
    ) -> VmResult<Value> {
        let result = self.call_function(store, name, args)?;
        let expected = 1;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(result[0].clone())
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

    /// Increments the call depth by 1 and returns the new value
    pub fn increment_call_depth(&self) -> VmResult<usize> {
        let new = self.with_context_data_mut(|context_data| {
            let new = context_data.call_depth + 1;
            context_data.call_depth = new;
            new
        });
        if new > MAX_CALL_DEPTH {
            return Err(VmError::max_call_depth_exceeded());
        }
        Ok(new)
    }

    /// Decrements the call depth by 1 and returns the new value
    pub fn decrement_call_depth(&self) -> usize {
        self.with_context_data_mut(|context_data| {
            let new = context_data
                .call_depth
                .checked_sub(1)
                .expect("Call depth < 0. This is a bug.");
            context_data.call_depth = new;
            new
        })
    }

    /// Returns the remaining gas measured in [CosmWasm gas].
    ///
    /// [CosmWasm gas]: https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md
    pub fn get_gas_left(&self, store: &mut impl AsStoreMut) -> u64 {
        self.with_wasmer_instance(|instance| {
            Ok(match get_remaining_points(store, instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            })
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    /// Sets the remaining gas measured in [CosmWasm gas].
    ///
    /// [CosmWasm gas]: https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md
    pub fn set_gas_left(&self, store: &mut impl AsStoreMut, new_value: u64) {
        self.with_wasmer_instance(|instance| {
            set_remaining_points(store, instance, new_value);
            Ok(())
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    /// Decreases gas left by the given amount.
    /// If the amount exceeds the available gas, the remaining gas is set to 0 and
    /// an VmError::GasDepletion error is returned.
    #[allow(unused)] // used in tests
    pub fn decrease_gas_left(&self, store: &mut impl AsStoreMut, amount: u64) -> VmResult<()> {
        self.with_wasmer_instance(|instance| {
            let remaining = match get_remaining_points(store, instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            };
            if amount > remaining {
                set_remaining_points(store, instance, 0);
                Err(VmError::gas_depletion())
            } else {
                set_remaining_points(store, instance, remaining - amount);
                Ok(())
            }
        })
    }

    /// Creates a MemoryView.
    /// This must be short living and not be used after the memory was grown.
    pub fn memory<'a>(&self, store: &'a impl AsStoreMut) -> MemoryView<'a> {
        self.memory
            .as_ref()
            .expect("Memory is not set. This is a bug in the lifecycle.")
            .view(store)
    }

    /// Moves owned instances of storage and querier into the env.
    /// Should be followed by exactly one call to move_out when the instance is finished.
    pub fn move_in(&self, storage: S, querier: Q) {
        self.with_context_data_mut(|context_data| {
            context_data.storage = Some(storage);
            context_data.querier = Some(querier);
        });
    }

    /// Returns the original storage and querier as owned instances, and closes any remaining
    /// iterators. This is meant to be called when recycling the instance.
    pub fn move_out(&self) -> (Option<S>, Option<Q>) {
        self.with_context_data_mut(|context_data| {
            (context_data.storage.take(), context_data.querier.take())
        })
    }
}

pub struct ContextData<S, Q> {
    gas_state: GasState,
    storage: Option<S>,
    storage_readonly: bool,
    call_depth: usize,
    querier: Option<Q>,
    debug_handler: Option<Rc<RefCell<DebugHandlerFn>>>,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
}

impl<S: Storage, Q: Querier> ContextData<S, Q> {
    pub fn new(gas_limit: u64) -> Self {
        ContextData::<S, Q> {
            gas_state: GasState::with_limit(gas_limit),
            storage: None,
            storage_readonly: true,
            call_depth: 0,
            querier: None,
            debug_handler: None,
            wasmer_instance: None,
        }
    }
}

pub fn process_gas_info<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    store: &mut impl AsStoreMut,
    info: GasInfo,
) -> VmResult<()> {
    let gas_left = env.get_gas_left(store);

    let new_limit = env.with_gas_state_mut(|gas_state| {
        gas_state.externally_used_gas += info.externally_used;
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        gas_left
            .saturating_sub(info.externally_used)
            .saturating_sub(info.cost)
    });

    // This tells wasmer how much more gas it can consume from this point in time.
    env.set_gas_left(store, new_limit);

    if info.externally_used + info.cost > gas_left {
        Err(VmError::gas_depletion())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::ref_to_u32;
    use crate::size::Size;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::wasm_backend::{compile, make_compiling_engine};
    use cosmwasm_std::{
        coin, coins, from_json, to_json_vec, BalanceResponse, BankQuery, Empty, QueryRequest,
    };
    use wasmer::{imports, Function, Instance as WasmerInstance, Store};

    static HACKATOM: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    // prepared data
    const INIT_KEY: &[u8] = b"foo";
    const INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const TESTING_GAS_LIMIT: u64 = 500_000_000; // ~0.5ms
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));

    fn make_instance(
        gas_limit: u64,
    ) -> (
        Environment<MockApi, MockStorage, MockQuerier>,
        Store,
        Box<WasmerInstance>,
    ) {
        let env = Environment::new(MockApi::default(), gas_limit);

        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&engine, HACKATOM).unwrap();
        let mut store = Store::new(engine);

        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_typed(&mut store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_typed(&mut store, |_a: u32| {}),
                "db_scan" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_key" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_value" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_validate" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_canonicalize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "addr_humanize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "bls12_381_aggregate_g1" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "bls12_381_aggregate_g2" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "bls12_381_pairing_equality" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32, _d: u32| -> u32 { 0 }),
                "bls12_381_hash_to_g1" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32, _d: u32| -> u32 { 0 }),
                "bls12_381_hash_to_g2" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32, _d: u32| -> u32 { 0 }),
                "secp256k1_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256k1_recover_pubkey" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "secp256r1_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256r1_recover_pubkey" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "ed25519_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "ed25519_batch_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "debug" => Function::new_typed(&mut store, |_a: u32| {}),
                "abort" => Function::new_typed(&mut store, |_a: u32| {}),
            },
        };
        let instance = Box::from(WasmerInstance::new(&mut store, &module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(&mut store, gas_limit);

        (env, store, instance)
    }

    fn leave_default_data(env: &Environment<MockApi, MockStorage, MockQuerier>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(INIT_ADDR, &coins(INIT_AMOUNT, INIT_DENOM))]);
        env.move_in(storage, querier);
    }

    #[test]
    fn move_out_works() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);

        // empty data on start
        let (inits, initq) = env.move_out();
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&env);
        let (s, q) = env.move_out();
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = env.move_out();
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    fn process_gas_info_works_for_cost() {
        let (env, mut store, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(&mut store), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, &mut store, GasInfo::with_cost(70)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 30);
        process_gas_info(&env, &mut store, GasInfo::with_cost(4)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 26);
        process_gas_info(&env, &mut store, GasInfo::with_cost(6)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 20);
        process_gas_info(&env, &mut store, GasInfo::with_cost(20)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, &mut store, GasInfo::with_cost(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn process_gas_info_works_for_externally_used() {
        let (env, mut store, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(&mut store), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(70)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 30);
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 26);
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 20);
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, &mut store, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn process_gas_info_works_for_cost_and_externally_used() {
        let (env, mut store, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(&mut store), 100);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 0);

        process_gas_info(&env, &mut store, GasInfo::new(17, 4)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 79);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, &mut store, GasInfo::new(9, 0)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 70);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, &mut store, GasInfo::new(0, 70)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More cost fail but do not change stats
        match process_gas_info(&env, &mut store, GasInfo::new(1, 0)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {err:?}"),
        }
        assert_eq!(env.get_gas_left(&mut store), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More externally used fails and changes stats
        match process_gas_info(&env, &mut store, GasInfo::new(0, 1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {err:?}"),
        }
        assert_eq!(env.get_gas_left(&mut store), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 75);
    }

    #[test]
    fn process_gas_info_zeros_gas_left_when_exceeded() {
        // with_externally_used
        {
            let (env, mut store, _instance) = make_instance(100);
            let result = process_gas_info(&env, &mut store, GasInfo::with_externally_used(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {err:?}"),
            }
            assert_eq!(env.get_gas_left(&mut store), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 120);
        }

        // with_cost
        {
            let (env, mut store, _instance) = make_instance(100);
            let result = process_gas_info(&env, &mut store, GasInfo::with_cost(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {err:?}"),
            }
            assert_eq!(env.get_gas_left(&mut store), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 0);
        }
    }

    #[test]
    fn process_gas_info_works_correctly_with_gas_consumption_in_wasmer() {
        let (env, mut store, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(&mut store), 100);

        // Some gas was consumed externally
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(50)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 50);
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 46);

        // Consume 20 gas directly in wasmer
        env.decrease_gas_left(&mut store, 20).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 26);

        process_gas_info(&env, &mut store, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 20);
        process_gas_info(&env, &mut store, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(&mut store), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, &mut store, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        assert!(env.is_storage_readonly());
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        // change
        env.set_storage_readonly(false);
        assert!(!env.is_storage_readonly());

        // still false
        env.set_storage_readonly(false);
        assert!(!env.is_storage_readonly());

        // change back
        env.set_storage_readonly(true);
        assert!(env.is_storage_readonly());
    }

    #[test]
    fn call_function_works() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env
            .call_function(&mut store, "allocate", &[10u32.into()])
            .unwrap();
        let ptr = ref_to_u32(&result[0]).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function_fails_for_missing_instance() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        // Clear context's wasmer_instance
        env.set_wasmer_instance(None);

        let res = env.call_function(&mut store, "allocate", &[]);
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn call_function_fails_for_missing_function() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let res = env.call_function(&mut store, "doesnt_exist", &[]);
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(msg, "Could not get export: Missing export doesnt_exist");
            }
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn call_function0_works() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.call_function0(&mut store, "interface_version_8", &[])
            .unwrap();
    }

    #[test]
    fn call_function0_errors_for_wrong_result_count() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env.call_function0(&mut store, "allocate", &[10u32.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "allocate");
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            err => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn call_function1_works() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env
            .call_function1(&mut store, "allocate", &[10u32.into()])
            .unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function1_errors_for_wrong_result_count() {
        let (env, mut store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env
            .call_function1(&mut store, "allocate", &[10u32.into()])
            .unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);

        let result = env.call_function1(&mut store, "deallocate", &[ptr.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "deallocate");
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            err => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
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
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.with_storage_from_context::<_, ()>(|_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    #[allow(deprecated)]
    fn with_querier_from_context_works() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let res = env
            .with_querier_from_context::<_, _>(|querier| {
                let req: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::Balance {
                    address: INIT_ADDR.to_string(),
                    denom: INIT_DENOM.to_string(),
                });
                let (result, _gas_info) =
                    querier.query_raw(&to_json_vec(&req).unwrap(), DEFAULT_QUERY_GAS_LIMIT);
                Ok(result.unwrap())
            })
            .unwrap()
            .unwrap()
            .unwrap();
        let balance: BalanceResponse = from_json(res).unwrap();

        assert_eq!(balance.amount, coin(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_querier_from_context_handles_panics() {
        let (env, _store, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.with_querier_from_context::<_, ()>(|_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }
}
