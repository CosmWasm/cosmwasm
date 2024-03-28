# Overview
**Capabilities** in CosmosWasm are a fundamental concept, designed to facilitate the interaction between a smart contract and its operational environment (i.e., the blockchain implementing the cosmwasm-vm or [wasmvm]). This mechanism operates by matching the capabilities required by a contract with those offered by the environment.

# The Essence of Capabilities
**Purpose:** They serve as a negotiation tool, ensuring that a contract only operates in an environment where all its necessary functionalities are present.
**Early Detection:** By verifying capabilities when a contract is first stored, any discrepancies or missing capabilities are identified early. This preemptive approach prevents potential runtime failures during execution.

# Historical Context: Origin and Disambiguation
**Prior to August 2022:** The term "features" was used ambiguously to describe both app-level features in the CosmWasm VM and features in Cargo's build system.
+**Redefinition:** To mitigate this confusion, app-level features in the CosmWasm VM were renamed to "capabilities", distinguishing them from Cargo's build system features.
+**Language Independence:** Unlike Rust-specific features, capabilities can be implemented in any language that compiles to Wasm, broadening their applicability.

# Defining Required Capabilities
## Implementation
**Marker Export Functions:** Contracts specify required capabilities using these functions, which have no arguments and return no value.
**Naming Convention:** The export name begins with "requires\_" followed by the capability name, establishing a clear and standardized method for declaration.

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

As mentioned above, the Cargo features are independent of the capabilities we
talk about and it is perfectly fine to have a requires\_\* export that is
unconditional in a library or a contract.

The marker export functions can be executed, but the VM does not require such a
call to succeed. So a contract can use no-op implementation or crashing
implementation.

## Available capabilities

An instance of the main `Cache` has `available_capabilities` in its
`CacheOptions`. This value is set in the caller, such as
[here](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-rc.0/libwasmvm/src/cache.rs#L75)
and
[here](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-rc.0/libwasmvm/src/cache.rs#L62).
`capabilities_from_csv` takes a comma separated list and returns a set of
capabilities. This capabilities list is set
[in keeper.go](https://github.com/CosmWasm/wasmd/blob/v0.27.0-rc0/x/wasm/keeper/keeper.go#L100)
and
[in app.go](https://github.com/CosmWasm/wasmd/blob/v0.27.0-rc0/app/app.go#L475-L496).

## Format

The capability name needs to be allowed as a Wasm export names and be a legal
function name in Rust and other CosmWasm smart contract languages such as Go. By
convention, the name should be short and all lower ASCII alphanumerical plus
underscores.

## Built-in capabilities

Here is a list of all [built-in capabilities](CAPABILITIES-BUILT-IN.md).

## What's a good capability?

A good capability makes sense to be disabled. The examples above explain why the
capability is not present in some environments.

Also when the environment adds new functionality in a way that does not break
existing contracts (such as new queries), capabilities can be used to ensure the
contract checks the availability early on.

When functionality is always present in the VM (such as a new import implemented
directly in the VM, see [#1299]), we should not use capability. They just create
fragmentation in the CosmWasm ecosystem and increase the barrier for adoption.
Instead the `check_wasm_imports` check is used to validate this when the
contract is stored.

[wasmvm]: https://github.com/CosmWasm/wasmvm
[#1299]: https://github.com/CosmWasm/cosmwasm/pull/1299
