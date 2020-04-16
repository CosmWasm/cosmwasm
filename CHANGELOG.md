# CHANGELOG

## 0.8.0 (not yet released)

**all**

- Upgrade schemars to 0.7.
- Upgrade wasmer to 0.16.
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
- Add new `ApiResult` and `ApiError` types to represent serializable
  counterparts to `errors::Result` and `errors::Error`. There are conversions
  from `Error` into serializable `ApiError` (dropping runtime info), and back
  and forth between `Result` and `ApiResult` (with the same serializable error
  types).
- Add `Querier` trait and `QueryRequest` for future query callbacks from the
  contract, along with `SystemError` type for the runtime rejecting messages.
- `{Init,Handle,Query}Result` are now just aliases for a concrete `ApiResult`
  type.
- Support results up to 128 KiB in `ExternalStorage.get`.
- The `Storage` trait's `.get`, `.set` and `.remove` now return a `Result` to
  allow propagation of errors.
- Move `transactional`, `transactional_deps`, `RepLog`, `StorageTransaction`
  into crate `cosmwasm-storage`.
- Rename `Result` to `StdResult` to differentiate between the auto-`use`d
  `core::result::Result`. Fix error argument to `Error`.
- Rename `Error` to `StdError`.

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
