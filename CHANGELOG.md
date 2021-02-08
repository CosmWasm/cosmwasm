# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- cosmwasm-vm: Add PinnedMemoryCache. ([#696])
- cosmwasm-vm: The new `Cache::analyze` provides a static analyzis of the Wasm
  bytecode. This is used to tell the caller if the contract exposes IBC entry
  points. ([#736])
- cosmwasm-vm: Added new `stargate` feature flag to enable new stargate and ibc
  features ([#692], [#716])
- cosmwasm-vm: (requires `stargate`) call into 6 new ibc entry points if exposed
  by contract ([#692], [#716])
- cosmwasm-std: Added new `stargate` feature flag to enable new stargate and ibc
  features ([#692], [#706])
- cosmwasm-std: (requires `stargate`) Added new `CosmosMsg::Stargate` message
  type to dispatch protobuf-encoded message (contract must know proto schema)
  ([#706])
- cosmwasm-std: (requires `stargate`) Added new `QueryRequest::Stargate` message
  type to dispatch protobuf-encoded queries (contract must know proto schema for
  request and response) ([#706])
- cosmwasm-std: (requires `stargate`) Added new `CosmosMsg::Ibc(IbcMsg)` message
  type to use ibctransfer app or send raw ics packets (if contract has ibc entry
  points) ([#692], [#710])
- cosmwasm-std: Add mutable helper methods to `InitResponse`, `MigrateResponse`
  and `HandleResponse` which make `Context` obsolete.
- contracts: added new `ibc-reflect` contract that receives channels and assigns
  each an account to redispatch. Similar idea to ICS27/Interchain Accounts (but
  different implementation) ([#692], [#711], [#714])
- cosmwasm-std: Added new `WasmMsg::Migrate` variant that allows one contract
  (eg. multisig) be the admin and migrate another contract ([#768])

[#692]: https://github.com/CosmWasm/cosmwasm/issues/692
[#706]: https://github.com/CosmWasm/cosmwasm/pull/706
[#710]: https://github.com/CosmWasm/cosmwasm/pull/710
[#711]: https://github.com/CosmWasm/cosmwasm/pull/711
[#714]: https://github.com/CosmWasm/cosmwasm/pull/714
[#716]: https://github.com/CosmWasm/cosmwasm/pull/716
[#768]: https://github.com/CosmWasm/cosmwasm/pull/768

### Changed

- all: The `query` enpoint is now optional. It is still highly recommended to
  expose it an almost any use case though.
- all: Change the encoding of the key/value region of the `db_next` import to a
  more generic encoding that supports an arbitrary number of sections. This
  encoding can then be reused for other multi value regions.
- all: Remove the `info: MessageInfo` argument from the `migrate` entry point
  ([#690]).
- cosmwasm-std: Remove `from_address` from `BankMsg::Send`, as it always sends
  from the contract address, and this is consistent with other `CosmosMsg`
  variants.
- cosmwasm-std: Remove the previously deprecated `InitResult`, `HandleResult`,
  `MigrateResult` and `QueryResult` in order to make error type explicit and
  encourage migration to custom errors.
- cosmwasm-std: Add a `data` field to `InitResponse` the same way as in
  `MigrateResponse` and `HandleResponse`.
- cosmwasm-std: Rename `MessageInfo::sent_funds` to `MessageInfo::funds`.
- cosmwasm-std: Merge response types `InitResponse`, `HandleResponse` and
  `MigrateResponse` into the new `Response`.
- cosmwasm-vm: Avoid serialization of Modules in `InMemoryCache`, for
  performance. Also, remove `memory_limit` from `InstanceOptions`, and define it
  instead at `Cache` level (same memory limit for all cached instances).
  ([#697])
- cosmwasm-vm: Bump required marker export `cosmwasm_vm_version_4` to
  `interface_version_5`.
- contracts: `reflect` contract requires `stargate` feature and supports
  redispatching `Stargate` and `IbcMsg::Transfer` messages ([#692])

[#696]: https://github.com/CosmWasm/cosmwasm/issues/696
[#697]: https://github.com/CosmWasm/cosmwasm/issues/697
[#736]: https://github.com/CosmWasm/cosmwasm/pull/736
[#690]: https://github.com/CosmWasm/cosmwasm/issues/690

### Deprecated

- cosmwasm-std: `InitResponse`, `MigrateResponse` and `HandleResponse` are
  deprecated in favour of the new `Response`.
- cosmwasm-std: `Context` is deprecated in favour of the new mutable helpers in
  `Response`.

## [0.13.2] - 2021-01-14

## Changed

- cosmwasm-vm: Update Wasmer to 1.0.1.

## [0.13.1] - 2021-01-12

### Added

- cosmwasm-std: Add the new `#[entry_point]` macro attribute that serves as an
  alternative implementation to `cosmwasm_std::create_entry_points!(contract)`
  and `cosmwasm_std::create_entry_points_with_migration!(contract)`. Both ways
  are supported in the 0.13 series.

## [0.13.0] – 2021-01-06

## Added

- cosmwasm-std: Extend binary to array support to 64 bytes.

## Changed

- all: Drop support for Rust versions lower than 1.47.0.
- cosmwasm-std: Remove `cosmwasm_std::testing::MockApi::new`. Use
  `MockApi::default` instead.
- cosmwasm-vm: Upgrade Wasmer to 1.0 and adapt all the internal workings
  accordingly.
- cosmwasm-vm: Export method `cosmwasm_vm::Cache::stats` and response type
  `Stats`.
- cosmwasm-vm: Remove `cosmwasm_vm::testing::MockApi::new`. Use
  `MockApi::default` instead.
- cosmwasm-vm: Convert field `Instance::api` to a method.
- cosmwasm-vm: Change order of generic arguments for consistency in `Instance`,
  `Cache` and `Backend` to always match `<A: Api, S: Storage, Q: Querier>`.
- cosmwasm-vm: Remove `Instance::get_memory_size`. Use `Instance::memory_pages`
  instead.

## 0.12.2 (2020-12-14)

**cosmwasm-std**

- `StdError` now implements `PartialEq` (ignoring backtrace if any). This allows
  simpler `assert_eq!()` when testing error conditions (rather than match
  statements as now).

## 0.12.1 (2020-12-09)

**cosmwasm-std**

- Deprecate `InitResult`, `HandleResult`, `MigrateResult` and `QueryResult` in
  order to make error type explicit and encourage migration to custom errors.
- Implement `Deref` for `QuerierWrapper`, such that `QuerierWrapper` behaves
  like a smart pointer to `Querier` allowing you to access `Querier` methods
  directly.

## 0.12.0 (2020-11-19)

**cosmwasm-std**

- Remove the previously deprecated `StdError::Unauthorized`. Contract specific
  errors should be implemented using custom error types now (see
  [migration guide](./MIGRATING.md) 0.10 -> 0.11).
- Use dependency `thiserror` instead of `snafu` to implement `StdError`. Along
  with this change, the `backtraces` feature now requires Rust nightly.
- Rename `StdError::ParseErr::source` to `StdError::ParseErr::source_type` and
  `StdError::SerializeErr::target` to `StdError::SerializeErr::target_type` to
  work around speacial treatment of the field name `source` in thiserror.
- Rename `Extern` to `Deps` to unify naming.
- Simplify ownership of calling `handle`, etc. with `Deps` and `DepsMut` struct
  that just contains references (`DepsMut` has `&mut Storage` otherwise the
  same)
- Remove unused `Deps::change_querier`. If you need this or similar
  functionality, create a new struct with the right querier.
- Remove `ReadonlyStorage`. You can just use `Storage` everywhere. And use
  `&Storage` to provide readonly access. This was only needed to let
  `PrefixedStorage`/`ReadonlyPrefixedStorage` implement the common interface,
  which is something we don't need.

**cosmwasm-storage**

- `PrefixedStorage`/`ReadonlyPrefixedStorage` do not implement the
  `Storage`/`ReadonlyStorage` traits anymore. If you need nested prefixes, you
  need to construct them directly via `PrefixedStorage::multilevel` and
  `ReadonlyPrefixedStorage::multilevel`.
- Remove unused `TypedStorage`. If you need this or similar functionality, you
  probably want to use `Bucket` or `Singleton`. If you really need it, please
  copy the v0.11 code into your project.
- Remove `StorageTransaction` along with `transactional` and `RepLog`. This has
  not been used actively for contract development and is now maintained in a
  contract testing framework.

**cosmwasm-vm**

- Remove `Storage::range` and `StorageIterator`. The storage implementation is
  now responsible for maintaining iterators internally and make them accessible
  via the new `Storage::scan` and `Storage::next` methods.
- Add `FfiError::IteratorDoesNotExist`. Looking at this, `FfiError` should
  probably be renamed to something that includes before, on and behind the FFI
  boundary to Go.
- `MockStorage` now implementes the new `Storage` trait and has an additional
  `MockStorage::all` for getting all elements of an iterator in tests.
- Remove unused `Extern::change_querier`. If you need this or similar
  functionality, create a new struct with the right querier.
- Let `Instance::from_code` and `CosmCache::get_instance` take options as an
  `InstanceOptions` struct. This contains `gas_limit` and `print_debug` for now
  and can easily be extended. `cosmwasm_vm::testing::mock_instance_options` can
  be used for creating such a struct in integration tests.
- Make `FileSystemCache` crate internal. This should be used via `CosmCache`.
- Fix return type of `FileSystemCache::load` to `VmResult<Option<Module>>` in
  order to differentiate missing files from errors.
- Add in-memory caching for recently used Wasm modules.
- Rename `CosmCache` to just `cosmwasm_vm::Cache` and add `CacheOptions` to
  configure it.
- Rename `Extern` to `Backend`.
- Rename `mock_dependencies` to `mock_backend` and
  `mock_dependencies_with_balances` to `mock_backend_with_balances`.
- Rename `FfiError`/`FfiResult` to `BackendError`/`BackendResult` and adapt
  `VmError` accordingly.

## 0.11.2 (2020-10-26)

**cosmwasm-std**

- Implement `From<std::str::Utf8Error>` and `From<std::string::FromUtf8Error>`
  for `StdError`.
- Generalize denom argument from `&str` to `S: Into<String>` in `coin`, `coins`
  and `Coin::new`.
- Implement `PartialEq` between `Binary` and `Vec<u8>`/`&[u8]`.
- Add missing `PartialEq` implementations between `HumanAddr` and `str`/`&str`.
- Add `Binary::to_array`, which allows you to copy binary content into a
  fixed-length `u8` array. This is espeically useful for creating integers from
  binary data.

## 0.11.1 (2020-10-12)

**cosmwasm-std**

- Implement `Hash` and `Eq` for `Binary` to allow using `Binary` in `HashSet`
  and `HashMap`.
- Implement `Hash` and `Eq` for `CanonicalAddr` to allow using `CanonicalAddr`
  in `HashSet` and `HashMap`.
- Implement `Add`, `AddAssign` and `Sub` with references on the right hand side
  for `Uint128`.
- Implement `Sum<Uint128>` and `Sum<&'a Uint128>` for `Uint128`.

## 0.11.0 (2020-10-08)

**all**

- Drop support for Rust versions lower than 1.45.2.
- The serialization of the result from `init`/`migrate`/`handle`/`query` changed
  in an incompatible way. See the new `ContractResult` and `SystemResult` types
  and their documentation.
- Pass `Env` into `query` as well. As this doesn't have `MessageInfo`, we
  removed `MessageInfo` from `Env` and pass that as a separate argument to
  `init`, `handle`, and `query`. See the example
  [type definitions in the README](README.md#implementing-the-smart-contract) to
  see how to update your contract exports (just add one extra arg each).

**cosmwasm-std**

- Add `time_nanos` to `BlockInfo` allowing access to high precision block times.
- Change `FullDelegation::accumulated_rewards` from `Coin` to `Vec<Coin>`.
- Rename `InitResponse::log`, `MigrateResponse::log` and `HandleResponse::log`
  to `InitResponse::attributes`, `MigrateResponse::attributes` and
  `HandleResponse::attributes`.
- Rename `LogAttribute` to `Attribute`.
- Rename `log` to `attr`.
- Rename `Context::add_log` to `Context::add_attribute`.
- Add `Api::debug` for emitting debug messages during development.
- Fix error type for response parsing errors in `ExternalQuerier::raw_query`.
  This was `Ok(Err(StdError::ParseErr))` instead of
  `Err(SystemError::InvalidResponse)`, implying an error created in the target
  contract.
- Deprecate `StdError::Unauthorized` and `StdError::unauthorized` in favour of
  custom errors. From now on `StdError` should only be created by the standard
  library and should only contain cases the standard library needs.
- Let `impl Display for CanonicalAddr` use upper case hex instead of base64.
  This also affects `CanonicalAddr::to_string`.
- Create trait `CustomQuery` for the generic argument in
  `QueryRequest<C: CustomQuery>`. This allows us to provide
  `impl<C: CustomQuery> From<C> for QueryRequest<C>` for any custom query.
- Implement `From<Binary> for Vec<u8>`.
- Implement `From<CanonicalAddr> for Vec<u8>`.
- Add `Binary::into_vec` and `CanonicalAddr::into_vec`.
- The `canonical_length` argument was removed from `mock_dependencies`,
  `mock_dependencies_with_balances`. In the now deprecated `MockApi::new`, the
  argument is unused. Contracts should not need to set this value and usually
  should not make assumptions about the value.
- The canonical address encoding in `MockApi::canonical_address` and
  `MockApi::human_address` was changed to an unpredicatable represenation of
  non-standard length that aims to destroy most of the input structure.

**cosmwasm-storage**

- Change order of arguments such that `storage` is always first followed by
  namespace in `Bucket::new`, `Bucket::multilevel`, `ReadonlyBucket::new`,
  `ReadonlyBucket::multilevel`, `bucket` and `bucket_read`.
- Change order of arguments such that `storage` is always first followed by
  namespace in `PrefixedStorage::new`, `PrefixedStorage::multilevel`,
  `ReadonlyPrefixedStorage::new`, `ReadonlyPrefixedStorage::multilevel`,
  `prefixed` and `prefixed_read`.

**cosmwasm-vm**

- `CosmCache::new`, `Instance::from_code` and `Instance::from_module` now take
  an additional argument to enable/disable printing debug logs from contracts.
- Bump required export `cosmwasm_vm_version_3` to `cosmwasm_vm_version_4`.
- The `canonical_length` argument was removed from `mock_dependencies`,
  `mock_dependencies_with_balances` and `MockApi::new_failing`. In the now
  deprecated `MockApi::new`, the argument is unused. Contracts should not need
  to set this value and usually should not make assumptions about the value.
- The canonical address encoding in `MockApi::canonical_address` and
  `MockApi::human_address` was changed to an unpredicatable represenation of
  non-standard length that aims to destroy most of the input structure.

## 0.10.1 (2020-08-25)

**cosmwasm-std**

- Fix bug where `ExternalStorage.range()` would cause VM error if either lower
  or upper bound was set
  ([#508](https://github.com/CosmWasm/cosmwasm/issues/508))

## 0.10.0 (2020-07-30)

**all**

- Drop support for Rust versions lower than 1.44.1.

**cosmwasm-std**

- Remove error helpers `generic_err`, `invalid_base64`, `invalid_utf8`,
  `not_found`, `parse_err`, `serialize_err`, `underflow`, `unauthorized` in
  favour of `StdError::generic_err` and friends.
- Implement `From<&[u8; $N]> for Binary` and `From<[u8; $N]> for Binary` for all
  `$N <= 32`.
- Add `Context` object that can be used to build Init/Handle/Migrate response
  via `add_log`, `add_message`, `set_data` and then convert to the proper type
  via `into` or `try_into`. Option to simplify response construction.
- Env uses `HumanAddr` for `message.sender` and `contract_address`
- Remove `Api` argument from `mock_env`
- Implement `From<&[u8]>` and `From<Vec<u8>>` for `CanonicalAddr`

**cosmwasm-vm**

- Remove unused cache size argument from `CosmCache`.
- `set_gas_limit` now panics if the given gas limit exceeds the max. supported
  value.
- Increase the max. supported value for gas limit from 10_000_000_000 to
  0x7FFFFFFFFFFFFFFF.
- Add checks to `get_region` for failing early when the contract sends a Region
  pointer to the VM that is not backed by a plausible Region. This helps
  development of standard libraries.
- Create dedicated `RegionValidationError` and `RegionValidationResult`.
- `Api::human_address` and `Api::canonical_address` now return a pair of return
  data and gas usage.
- Remove `NextItem` in favour of a more advanced `FfiResult<T>`, which is used
  to store the return result and the gas information consistently across all
  APIs. `FfiResult<T>` was changed to `(Result<T, FfiError>, GasInfo)`.
- Create error type `FfiError::InvalidUtf8` for the cases where the backend
  sends invalid UTF-8 in places that expect strings.
- Remove `FfiError::Other` in favour of `FfiError::UserErr` and
  `FfiError::Unknown`.
- The `canonicalize_address` and `humanize_address` imports now report user
  errors to the contract.
- Bump `cosmwasm_vm_version_2` to `cosmwasm_vm_version_3`.
- `Querier::raw_query` and `QuerierResult` were removed in favour of the new
  `Querier::query_raw`, which includes a gas limit parameter for the query.

## 0.9.4 (2020-07-16)

**cosmwasm-vm**

- Add `Instance::create_gas_report` returning a gas report including the
  original limit, the remaining gas and the internally/externally used gas.

## 0.9.3 (2020-07-08)

**cosmwasm-storage**

- Add `.remove()` method to `Bucket` and `Singleton`.

## 0.9.2 (2020-06-29)

- Downgrade wasmer to 0.17.0.

## 0.9.1 (2020-06-25)

**cosmwasm-std**

- Replace type `Never` with `Empty` because enums with no cases cannot be
  expressed in valid JSON Schema.

## 0.9.0 (2020-06-25)

Note: this version contains an API bug and should not be used (see
https://github.com/CosmWasm/cosmwasm/issues/451).

**all**

- Upgrade wasmer to 0.17.1.
- Drop support for Rust versions lower than 1.43.1

**cosmwasm-std**

- `ReadonlyStorage::get` and all its implementations now return
  `Option<Vec<u8>>`.
- `ReadonlyStorage::range` and all its implementations now always succeed and
  return an iterator instead of a result. This is now an iterator over
  `Option<KV>` instead of `Option<StdResult<KV>>`.
- `Storage::{set, remove}` and all their implementations no longer have a return
  value. Previously they returned `StdResult<()>`.
- Trait `Querier` is not `Clone` and `Send` anymore.
- `consume_region` panics on null pointers and returns `Vec<u8>` instead of
  `StdResult<Vec<u8>>`.
- Added contract migration mechanism. Contracts can now optionally export a
  `migrate` function with the following definition:
  ```rust
  extern "C" fn migrate(env_ptr: u32, msg_ptr: u32) -> u32;
  ```
- InitResponse no longer has a data field. We always return the contract address
  in the data field in the blockchain and don't allow you to override. `handle`
  can still make use of the field.
- Rename `MockQuerier::with_staking` to `MockQuerier::update_staking` to match
  `::update_balance`.
- The obsolete `StdError::NullPointer` and `null_pointer` were removed.
- Error creator functions are now in type itself, e.g.
  `StdError::invalid_base64` instead of `invalid_base64`. The free functions are
  deprecated and will be removed before 1.0.

**cosmwasm-storage**

- Remove `transactional_deps`. Use `transactional` that just provides a
  transactional storage instead.
- `get_with_prefix` returns `Option<Vec<u8>>` instead of
  `StdResult<Option<Vec<u8>>>`.
- `set_with_prefix` and `remove_with_prefix` return nothing instead of
  `StdResult<()>`.
- `RepLog::commit` no longer returns any value (always succeeds).
- `Op::apply` no longer returns any value (always succeeds).

**cosmwasm-vm**

- The export `allocate` must not return 0 as a valid address. The contract is
  responsible for avoiding this offset in the linear memory.
- The import `db_read` now allocates memory for the return value as part of the
  call and returns a pointer to the value as `u32`. The return value 0 means
  _key does not exist_.
- The import `db_next` now allocates a memory region for the return key and
  value as part of the call and returns a pointer to the region as `u32`. The
  data in the region is stored in the format `value || key || keylen`. As
  before, an empty key means _no more value_.
- Remove `Instance::get_gas` in favour of `Instance::get_gas_left`.
- All calls from the VM layer to the chain layer also return the amount of gas
  used on success. (This is represented by replacing the return value with
  `(value, used_gas)`). Gas usage across the system is then tracked in the VM
  layer, which allows us to halt the contract during an import, as soon as we
  can prove that we used all allocated gas.
- Remove instance caching, which is disabled since 0.8.1 as it is not stable.
  Remove `CosmCache::store_instance`; you can not call `Instance::recylce`
  directly to get back the external dependencies.
- Rename `MockQuerier::with_staking` to `MockQuerier::update_staking` to match
  `::update_balance`.
- Instead of panicking, `read_region`/`write_region`/`get_region`/`set_region`
  now return a new `CommunicationError::DerefErr` when dereferencing a pointer
  provided by the contract fails.
- `FfiError::set_message` was removed because errors should be immutable. Use
  `FfiError::other` to create an error with the desired error message.
- The import implementation of `db_scan` now errors instead of returning an
  error code for an invalid order value. The return type was changed to `u32`.
- Remove `StorageIteratorItem` in favour of the new types `StorageIterator` and
  `NextItem`. `StorageIterator` is a custom iterator type that does not
  implement Rust's `Iterator` trait, allowing it to communicate the used gas
  value of the last `next` call to the VM.
- Don't report any `VmError` back to the contract in `canonicalize_address` and
  `humanize_address`. Only invalid inputs should be reported.
- Move error cases `VmError::RegionLengthTooBig` and `VmError::RegionTooSmall`
  into `CommunicationError`.
- In the `canonicalize_address` inplementation, invalid UTF-8 inputs now result
  in `CommunicationError::InvalidUtf8`, which is not reported back to the
  contract. A standard library should ensure this never happens by correctly
  encoding string input values.
- Merge trait `ReadonlyStorage` into `Storage`.
- The imports `canonicalize_address` and `humanize_address` now return a memory
  address to an error `Region`. If this address is 0, the call succeeded.
- Bump `cosmwasm_vm_version_1` to `cosmwasm_vm_version_2`.

## 0.8.1 (2020-06-08)

**cosmwasm-std**

- The arguments of `log` changed from `&str` to `ToString`, allowing to pass
  various types like `String`, `HumanAddr`, `Uint128` or primitive integers
  directly.
- Add `From<Vec<u8>>` and `Into<Vec<u8>>` implementations for `Binary` for
  zero-copy conversions.

**cosmwasm-vm**

- Deprecated `Instance::get_gas` in favour of `Instance::get_gas_left`. The old
  method will remain available for a while but will issue a deprecation warning
  when used.
- Disable instance caching by treating every cache size as 0. Instance caching
  is not safe as the same Wasm memory is reused across multiple executions.
- The storage of an `Instance` can now be set into readonly mode, which is
  checked by the writing storage imports `db_write` and `db_remove`. Read-only
  mode is off by default for backwards compatibility. `call_query_raw` now sets
  the instance's storage to readonly.
- The new error case `VmError::WriteAccessDenied` is returned when a contract
  calls an import that potentially writes to storage during a query.

## 0.8.0 (2020-05-25)

**all**

- Upgrade schemars to 0.7.
- Upgrade wasmer to 0.17.
- Update snafu to 0.6.
- Minimal supported Rust version is 1.41.
- Split `Region.len` into `Region.capacity` and `Region.length`, where the new
  capacity is the number of bytes available and `length` is the number of bytes
  used. This is a breaking change in the contract-vm interface, which requires
  the same memory layout of the `Region` struct on both sides.
- Add `remove` method to `Storage` trait.
- (feature-flagged) Add `range` method to `ReadonlyStorage` trait. This returns
  an iterator that covers all or a subset of the items in the db ordered
  ascending or descending by key.
- Add new feature flag `iterator` to both packages to enable `range`
  functionality. This is used to allow potential porting to chains that use
  Merkle Tries (which don't allow iterating over ranges).
- All serialized JSON types now use snake_case mappings for names. This means
  enum fields like `ChangeOwner` will map to `change_owner` in the underlying
  JSON, not `changeowner`. This is a breaking change for the clients.
- Public interface between contract and runtime no longer uses `String` to
  represent an error, but rather serializes `ApiError` as a rich JSON error.
- Return value from `env.write_db` and `env.remove_db` to allow error reporting.
- Query responses are now required to contain valid JSON.
- Renamed all `*_db` wasm imports to `db_*`
- Merge `cw-storage` repo as subpackage, now `cosmwasm-storage`
- Add iterator support to `cosmwasm-storage`
- `Coin.amount` is now `Uint128` rather than `String`. Uint128 serializes as a
  string in JSON, but parses into a u128 data in memory. It also has some
  operator overloads to allow easy math operations on `Coin` types, as well as
  enforcing valid amounts.
- `Env` no longer has a `contract.balance` element. If you need this info,
  please use the `Querier` to get this info. As of Cosmos-SDK 0.39 this needs
  extra storage queries to get the balance, so we only do those queries when
  needed.
- `Env.message.sent_funds` is a `Vec<Coin>` not `Option<Vec<Coin>>`. We will
  normalize the go response in `go-cosmwasm` before sending it to the contract.
- `Env.message.signer` was renamed to `Env.message.sender`.
- `Env.block.{height,time}` are now `u64` rather than `i64`.

**cosmwasm-schema**

- This new crate now contains the implementations for generating JSON Schema
  files from interface types. It exposes the functions `export_schema`,
  `export_schema_with_title`, and `schema_for`.

**cosmwasm-std**

- Make all symbols from `cosmwasm::memory` crate internal, as those symbols are
  not needed by users of the library.
- Rename `cosmwasm::mock::dependencies` -> `cosmwasm::mock::mock_dependencies`
  to differentiate between testing and production `External`.
- Export all symbols from `cosmwasm::mock` as the new non-wasm32 module
  `cosmwasm::testing`. Export all remaining symbols at top level (e.g.
  `use cosmwasm::traits::{Api, Storage};` + `use cosmwasm::encoding::Binary;`
  becomes `use cosmwasm::{Api, Binary, Storage};`).
- Rename package `cosmwasm` to `cosmwasm-std`.
- The export `allocate` does not zero-fill the allocated memory anymore.
- Add `remove_db` to the required imports of a contract.
- (feature-flagged) add `scan_db` and `next_db` callbacks from wasm contract to
  VM.
- `serde::{from_slice, to_vec}` return `cosmwasm_std::Result`, no more need to
  use `.context(...)` when calling these functions
- Split `Response` into `InitResponse` and `HandleResponse`; split
  `ContractResult` into `InitResult` and `HandleResult`.
- Create explicit `QueryResponse`, analogue to `InitResponse` and
  `HandleResponse`.
- The exports `cosmwasm_vm_version_1`, `allocate` and `deallocate` are now
  private and can only be called via the Wasm export. Make sure to `use`
  `cosmwasm_std` at least once in the contract to pull in the C exports.
- Add `Querier` trait and `QueryRequest` for query callbacks from the contract,
  along with `SystemError` type for the runtime rejecting messages.
- `QueryRequest` takes a generic `Custom(T)` type that is passed opaquely to the
  end consumer (`wasmd` or integration test stubs), allowing custom queries to
  native code.
- `{Init,Handle,Query}Result` are now just aliases for a concrete `ApiResult`
  type.
- Support results up to 128 KiB in `ExternalStorage.get`.
- The `Storage` trait's `.get`, `.set` and `.remove` now return a `Result` to
  allow propagation of errors.
- Move `transactional`, `transactional_deps`, `RepLog`, `StorageTransaction`
  into crate `cosmwasm-storage`.
- Rename `Result` to `StdResult` to differentiate between the auto-`use`d
  `core::result::Result`. Fix error argument to `Error`.
- Complete overhaul of `Error` into `StdError`:
  - The `StdError` enum can now be serialized and deserialized (losing its
    backtrace), which allows us to pass them over the Wasm/VM boundary. This
    allows using fully structured errors in e.g. integration tests.
  - Auto generated snafu error constructor structs like `NotFound`/`ParseErr`/…
    have been intenalized in favour of error generation helpers like
    `not_found`/`parse_err`/…
  - All error generator functions now return errors instead of results.
  - Error cases don't contain `source` fields anymore. Instead source errors are
    converted to standard types like `String`. For this reason, both
    `snafu::ResultExt` and `snafu::OptionExt` cannot be used anymore.
  - Backtraces became optional. Use `RUST_BACKTRACE=1` to enable them.
  - `Utf8Err`/`Utf8StringErr` merged into `StdError::InvalidUtf8`
  - `Base64Err` renamed into `StdError::InvalidBase64`
  - `ContractErr`/`DynContractErr` merged into `StdError::GeneralErr`
  - The unused `ValidationErr` was removed
  - `StdError` is now
    [non_exhaustive](https://doc.rust-lang.org/1.35.0/unstable-book/language-features/non-exhaustive.html),
    making new error cases non-breaking changes.
- `ExternalStorage.get` now returns an empty vector if a storage entry exists
  but has an empty value. Before, this was normalized to `None`.
- Reorganize `CosmosMsg` enum types. They are now split by modules:
  `CosmosMsg::Bank(BankMsg)`, `CosmosMsg::Custom(T)`, `CosmosMsg::Wasm(WasmMsg)`
- CosmosMsg is now generic over the content of `Custom` variant. This allows
  blockchains to support custom native calls in their Cosmos-SDK apps and
  developers to make use of them in CosmWasm apps without forking the
  `cosmwasm-vm` and `go-cosmwasm` runtime.
- Add `staking` feature flag to expose new `StakingMsg` types under `CosmosMsg`
  and new `StakingRequest` types under `QueryRequest`.
- Add support for mocking-out staking queries via `MockQuerier.with_staking`
- `from_slice`/`from_binary` now require result type to be `DeserializeOwned`,
  i.e. the result must not contain references such as `&str`.

**cosmwasm-vm**

- Make `Instance.memory`/`.allocate`/`.deallocate`/`.func` crate internal. A
  user of the VM must not access the instance's memory directly.
- The imports `env.canonicalize_address`, `env.humanize_address` and
  `env.read_db` don't return the number of bytes written anymore. This value is
  now available in the resulting regions. Negative return values are errors, 0
  is success and values greater than 0 are reserved for future use.
- Change the required interface version guard export from `cosmwasm_api_0_6` to
  `cosmwasm_vm_version_1`.
- Provide implementations for `remove_db` and (feature-flagged) `scan_db` and
  `next_db`
- Provide custom `serde::{from_slice, to_vec}` implementation separate from
  `cosmwasm_std`, so we can return cosmwasm-vm specific `Result` (only used
  internally).
- `call_{init,handle,query}` and the `cosmwasm_vm::testing` wrappers return
  standard `Result` types now, eg. `Result<HandleResponse, ApiError>`.
- Add length limit when reading memory from the instance to protect against
  malicious contracts creating overly large `Region`s.
- Add `Instance.get_memory_size`, giving you the peak memory consumption of an
  instance.
- Remove `cosmwasm_vm::errors::CacheExt`.
- Move `cosmwasm_vm::errors::{Error, Result}` to
  `cosmwasm_vm::{VmError, VmResult}` and remove generic error type from result.
- The import `db_read` now returns an error code if the storage key does not
  exist. The latest standard library converts this error code back to a `None`
  value. This allows differentiating non-existent and empty storage entries.
- Make `Instance::from_module`, `::from_wasmer` and `::recycle` crate-internal.
- Create explicit, public `Checksum` type to identify Wasm blobs.
- `CosmCache::new` now takes supported features as an argument.
- Rename `VmError::RegionTooSmallErr` to `VmError::RegionTooSmall`.
- Rename `VmError::RegionLengthTooBigErr` to `VmError::RegionLengthTooBig`.
- Change property types to owned string in `VmError::UninitializedContextData`,
  `VmError::ConversionErr`, `VmError::ParseErr` and `VmError::SerializeErr`.
- Remove `VmError::IoErr` in favour of `VmError::CacheErr`.
- Simplify `VmError::CompileErr`, `VmError::ResolveErr` and
  `VmError::WasmerRuntimeErr` to just hold a string with the details instead of
  the source error.
- Remove `VmError::WasmerErr` in favour of the new `VmError::InstantiationErr`.
- The snafu error builders from `VmError` are now private, i.e. callers can only
  use the errors, not create them.
- `VmError` is now `#[non_exhaustive]`.
- Split `VmError::RuntimeErr` in `VmError::BackendErr` and
  `VmError::GenericErr`; rename `VmError::WasmerRuntimeErr` to
  `VmError::RuntimeErr`.
- Add `Instance.with_querier` analogue to `Instance.with_storage`.

## 0.7.2 (2020-03-23)

**cosmwasm**

- Fix JSON schema type of `Binary` from int array (wrong) to string (right).
- Make `Extern` not `Clone`able anymore. Before cloning led to copying the data
  for mock storage and copying a stateless bridge for the external storage,
  which are different semantics.
- Remove public `cosmwasm::imports::dependencies`. A user of this library does
  not need to call this explicitely. Dependencies are created internally and
  passed as an argument in `exports::do_init`, `exports::do_handle` and
  `exports::do_query`.
- Make `ExternalStorage` not `Clone`able anymore. This does not copy any data,
  so a clone could lead to unexpected results.

## 0.7.1 (2020-03-11)

**cosmwasm_vm**

- Avoid unnecessary panic when checking corrupted wasm file.
- Support saving the same wasm to cache multiple times.

## 0.7.0 (2020-02-26)

**cosmwasm**

- Rename `Slice` to `Region` to simplify differentiation between Wasm memory
  region and serde's `from_slice`
- Rename `Params` to `Env`, `mock_params` to `mock_env` for clearer naming (this
  is information on the execution environment)
- `Response.log` is not a vector of key/value pairs that can later be indexed by
  Tendermint.

**cosmwasm_vm**

- Remove export `cosmwasm_vm::read_memory`. Using this indicates an
  architectural flaw, since this is a method for host to guest communication
  inside the VM and not needed for users of the VM.
- Create new type `cosmwasm_vm:errors::Error::RegionTooSmallErr`.
- Change return type of `cosmwasm_vm::write_memory` to `Result<usize, Error>` to
  make it harder to forget handling errors.
- Fix missing error propagation in `do_canonical_address`, `do_human_address`
  and `allocate`.
- Update error return codes in import `c_read`.
- Rename imports `c_read`/`c_write` to `read_db`/`write_db`.
- Rename imports `c_canonical_address`/`c_human_address` to
  `canonicalize_address`/`humanize_address`.
- Add `cosmwasm_vm::testing::test_io` for basic memory allocation/deallocation
  testing between host and guest.
- Make `ValidationErr.msg` a dynamic `String` including relevant runtime
  information.
- Remove export `check_api_compatibility`. The VM will take care of calling it.
- Let `check_api_compatibility` check imports by fully qualified identifier
  `<module>.<name>`.
- Make gas limit immutable in `cosmwasm_vm::instance::Instance`. It is passed
  once at construction time and cannot publicly be manipulated anymore.
- Remove `take_storage`/`leave_storage` from `cosmwasm_vm::Instance`.

## 0.6

[Define canonical address callbacks](https://github.com/confio/cosmwasm/issues/73)

- Use `&[u8]` for addresses in params
- Allow contracts to resolve human readable addresses (`&str`) in their json
  into a fixed-size binary representation
- Provide mocks for unit testing and integration tests

- Separate out `Storage` from `ReadOnlyStorage` as separate traits

## 0.5

### 0.5.2

This is the first documented and supported implementation. It contains the basic
feature set. `init` and `handle` supported for modules and can return messages.
A stub implementation of `query` is done, which is likely to be deprecated soon.
Some main points:

- The build-system and unit/integration-test setup is all stabilized.
- Cosmwasm-vm supports singlepass and cranelift backends, and caches modules on
  disk and instances in memory (lru cache).
- JSON Schema output works

All future Changelog entries will reference this base

[unreleased]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.1...HEAD
[0.13.2]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/CosmWasm/cosmwasm/compare/v0.12.0...v0.13.0
