# Features

Features are a mechanism to negotiate functionality between a contract and an
environment (i.e. the chain that embeds cosmwasm-vm/[wasmvm]) in a very
primitive way. The contract defines required features. The environment defines
supported features. If the required features are all supported, the contract can
be used. Doing this check when the contract is first stored ensures missing
features are detected early and not when a user tries to execute a certain code
path.

## Disambiguation

This document is about app level features in the CosmWasm VM. Features should
not be confused with Cargo's build system features, even when connected.
Features can be implemented in any language that compiles to Wasm.

## Required features

The contract defines required features using a marker export function that takes
no arguments and returns no value. The name of the export needs to start with
"requires\_" followed by the name of the feature. Do yourself a favor and keep
the name all lower ASCII alphanumerical.

An example of such markers in cosmwasm-std are those:

```rust
#[cfg(feature = "iterator")]
#[no_mangle]
extern "C" fn requires_iterator() -> () {}

#[cfg(feature = "staking")]
#[no_mangle]
extern "C" fn requires_staking() -> () {}

#[cfg(feature = "stargate")]
#[no_mangle]
extern "C" fn requires_stargate() -> () {}
```

which in Wasm compile to this:

```
# ...
  (export "requires_staking" (func 181))
  (export "requires_stargate" (func 181))
  (export "requires_iterator" (func 181))
# ...
  (func (;181;) (type 12)
    nop)
# ...
  (type (;12;) (func))
```

As mentioned above, the Cargo features are independent of the features we talk
about and it is perfectly fine to have a requires\_\* export that is
unconditional in a library or a contract.

The marker export functions can be executed, but the VM does not require such a
call to succeed. So a contract can use no-op implementation or crashing
implementation.

## Supported features

An instance of the main `Cache` has `supported_capabilities` in its
`CacheOptions`. This value is set in the caller, such as
[here](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-rc.0/libwasmvm/src/cache.rs#L75)
and
[here](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-rc.0/libwasmvm/src/cache.rs#L62).
`capabilities_from_csv` takes a comma separated list and returns a set of
features. This features list is set
[in keeper.go](https://github.com/CosmWasm/wasmd/blob/v0.27.0-rc0/x/wasm/keeper/keeper.go#L100)
and
[in app.go](https://github.com/CosmWasm/wasmd/blob/v0.27.0-rc0/app/app.go#L475-L496).

## Common features

Here is a list of features created in the past. Since features can be created
between contract and environment, we don't know them all in the VM.

- `iterator` is for storage backends that allow range queries. Not all types of
  databases do that. There are trees that don't allow it and Secret Network does
  not support iterators for other technical reasons.
- `stargate` is for messages and queries that came with the Cosmos SDK upgrade
  "Stargate". It primarily includes protobuf messages and IBC support.
- `staking` is for chains with the Cosmos SDK staking module. There are Cosmos
  chains that don't use this (e.g. Tgrade).

## What's a good feature?

A good feature makes sense to be disabled. The examples above explain why the
feature is not present in some environments.

When functionality is always present in the VM (such as a new import implemented
directly in the VM, see [#1299]), we should not use features. They just create
fragmentation in the CosmWasm ecosystem and increase the barrier for adoption.
Instead the `check_wasm_imports` check is used to validate this when the
contract is stored.

[wasmvm]: https://github.com/CosmWasm/wasmvm
[#1299]: https://github.com/CosmWasm/cosmwasm/pull/1299
