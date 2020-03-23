# CHANGELOG

## 0.8.0 (not yet released)

**cosmwasm**

- Make all symbols from `cosmwasm::memory` crate internal, as those symbols are
  not needed by users of the library.
- Export all symbols at top level (e.g.
  `use cosmwasm::traits::{Api, Storage};` + `use cosmwasm::encoding::Binary;`
  becomes `use cosmwasm::{Api, Binary, Storage};`).

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
