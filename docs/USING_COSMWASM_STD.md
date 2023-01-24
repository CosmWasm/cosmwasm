# Using cosmwasm-std

cosmwasm-std is the standard library for building contracts in CosmWasm. It is
compiled as part of the contract to Wasm. When creating a dependency to
cosmwasm-std, the required Wasm imports and exports are created implicitely via
C interfaces, e.g.:

```rust
// Exports
#[no_mangle]
extern "C" fn interface_version_8() -> () { /* ... */ }
#[no_mangle]
extern "C" fn allocate(size: usize) -> u32 { /* ... */ }
#[no_mangle]
extern "C" fn deallocate(pointer: u32) { /* ... */ }

// Imports
extern "C" {
    #[cfg(feature = "abort")]
    fn abort(source_ptr: u32);

    fn db_read(key: u32) -> u32;
    fn db_write(key: u32, value: u32);
    fn db_remove(key: u32);

    /* ... */
}
```

As those exports are not namespaced, only one version of cosmwasm-std must exist
in the dependency tree. Otherwise conflicting C exports are created.

## cosmwasm-std features

The libarary comes with the following features:

| Feature      | Enabled by default | Description                                                                |
| ------------ | ------------------ | -------------------------------------------------------------------------- |
| iterator     | x                  | Storage iterators                                                          |
| abort        | x                  | A panic handler that aborts the contract execution with a helpfull message |
| stargate     |                    | Cosmos SDK 0.40+ features and IBC                                          |
| ibc3         |                    | New fields added in IBC v3                                                 |
| staking      |                    | Access to the staking module                                               |
| baktraces    |                    | Add backtraces to errors (for unit testing)                                |
| cosmwasm_1_1 |                    | Features that require CosmWasm 1.1+ on the chain                           |
| cosmwasm_1_2 |                    | Features that require CosmWasm 1.2+ on the chain                           |

## The cosmwasm-std dependency for contract developers

As a contract developer you can simply specify the dependency as follows in
`Cargo.toml`:

```toml
cosmwasm-std = { version = "1.2.0" }
```

Please note that it is recommended to set all 3 version components and use the
minimum version you are willing to accept in the contract. For contracts this
would usually be the latest stable version.

Most likely you also want to enable the `stargate`, which is pretty basic these
days and maybe you know your chain supports CosmWasm 1.2 or higher. Then you add
those features:

```toml
cosmwasm-std = { version = "1.2.0", features = ["stargate", "cosmwasm_1_2"] }
```

## The cosmwasm-std dependency for library developers

When you are creating a library that uses cosmwasm-std, you should be incredibly
careful with which features you require. The moment you add e.g. `cosmwasm_1_2`
there it becomes impossible to use the contract in chains with lower CosmWasm
versions. If you add `abort`, it becomes impossible for the contract developer
to opt out of the abort feature due to your library. Since this affects the
default features `abort` and `iterator`, you should always disable default
features.

Also libraries should define a loose version range that allows the contract
developer to control which cosmwasm-std version they want to use in the final
project. E.g. if your library does not work with 1.0.0 due to a bug fixed in
1.0.1, your min version is 1.0.1 and not the latest stable.

A typical dependency then looks like this:

```toml
# We really need `stargate` here as this is an IBC related library. `abort` and `iterator` are not needed.
cosmwasm-std = { version = "1.0.1", default-features = false, features = ["stargate"] }
```
