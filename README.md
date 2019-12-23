# CosmWasm

[![CircleCI](https://circleci.com/gh/confio/cosmwasm/tree/master.svg?style=shield)](https://circleci.com/gh/confio/cosmwasm/tree/master) 
[![Docs](https://docs.rs/cosmwasm/badge.svg)](https://docs.rs/cosmwasm)
[![crates.io](https://img.shields.io/crates/v/cosmwasm.svg)](https://crates.io/crates/cosmwasm)


**Web Assembly Smart Contracts for the Cosmos SDK**

This repo provides a useful functionality to build smart contracts that
are compatible with Cosmos SDK runtime, [currently being developed](https://github.com/cosmwasm/cosmos-sdk/issues).

**Warning** Most likely you want to check out `v0.5.2` tag, the stable release referred to in the [documention](https://www.cosmwasm.com).
We are currently on `v0.6.0` with many breaking API changes, soon we will update the tutorials. (Along with a v0.7.0 release)

## Overview

This crate provides the bindings and all imports needed to build a smart contract.
However, to get that contract to interact with a system needs many moving parts.
To get oriented, here is a list of the various components of the CosmWasm ecosystem:

* [cosmwasm](https://github.com/confio/cosmwasm) - This crate. All needed functionality and no more - to build a small, efficient wasm smart contract.

**Building contracts:**

* [cosmwasm-template](https://github.com/confio/cosmwasm-template) - A starter-pack to get you quickly building your custom contract compatible with the cosmwasm system.
* [cosmwasm-examples](https://github.com/confio/cosmwasm-examples) - Some sample contracts (build with cosmwasm-template) for use and inspiration. Please submit your contract via PR.
* [cosmwasm-opt](https://github.com/confio/cosmwasm-opt) - A docker image and scripts to take your rust code and produce the smallest possible wasm output. *Deterministically*
This is designed both for preparing contracts for deployment as well as validating that a given deployed contract is based on some given rust code.,
allow a [similar contract verification algorithm](https://medium.com/coinmonks/how-to-verify-and-publish-on-etherscan-52cf25312945) as etherscan.
* [serde-json-wasm](https://github.com/confio/serde-json-wasm) - A custom json library, forked from `serde-json-core`. This provides an interface similar to
`serde-json`, but without ay floating-point instructions (non-deterministic) and producing builds
around 40% of the code size.

**Executing contracts:**

* [cosmwasm-vm](https://github.com/confio/cosmwasm/tree/master/lib/vm) - A sub-crate. Uses the [wasmer](https://github.com/wasmerio/wasmer) engine
to execute a given smart contract. Also contains code for gas metering, storing, and caching wasm artifacts. Read more [here](https://github.com/confio/cosmwasm/blob/master/lib/vm/README.md).
* [go-cosmwasm](https://github.com/confio/go-cosmwasm) - High-level go bindings to all the power inside `cosmwasm-vm`. Easily allows you to upload, instantiate and execute contracts,
making use of all the optimizations and caching available inside `cosmwasm-vm`.
* [Cosmos SDK](https://github.com/cosmwasm/modules/tree/master/incubator/contract) - Currently an WIP fork targeting `cosmos/modules` 
to provide an wasm module you can easily plug into any Cosmos-SDK based application. 
 
Ongoing work is currently tracked [on this project board](https://github.com/orgs/confio/projects/1)
for all of the internals, and [on this project board](https://github.com/cosmwasm/modules/projects/3)
for the Cosmos-SDK integration work.

## Creating a Smart Contract

You can see some examples of contracts under the `contracts` directory, 
which you can look at. 

If you want to get started building you own, the simplest
way is to go to the [cosmwasm-template](https://github.com/confio/cosmwasm-template)
repository and follow the instructions. This will give you a simple contract
along with tests, and a properly configured build environment. From there
you can edit the code to add your desired logic and publish it as an independent
repo.

If you want to understand a bit more, you can read some instructions on how 
we [configure a library for wasm](./Building.md)

## API entry points

Web Assembly contracts are basically black boxes. The have no default entry points,
and no access to the outside world by default. To make them useful, we need to add
a few elements. 

If you haven't worked with Web Assembly before, please read an overview
on [how to create imports and exports](./EntryPoints.md) in general.
 
The actual exports provided by the cosmwasm smart contract are:

```C
pub extern "C" fn init(params_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void;
pub extern "C" fn handle(params_ptr: *mut c_void, msg_ptr: *mut c_void) -> *mut c_void;

pub extern "C" fn allocate(size: usize) -> *mut c_void;
pub extern "C" fn deallocate(pointer: *mut c_void);
```
(`init` and `handle` must be defined by your contract. De-allocate can simply be
[re-exported exports.rs](https://github.com/confio/cosmwasm/blob/master/src/exports.rs#L16-L30))

And the imports provided to give you contract access to the environment are:

```C
extern "C" {
    fn c_read(key: *const c_void, value: *mut c_void) -> i32;
    fn c_write(key: *const c_void, value: *mut c_void);
}
```
(from [imports.rs](https://github.com/confio/cosmwasm/blob/master/src/imports.rs#L12-L17))

You could actually implement a Web Assembly module in any language, 
and as long as you implement these 6 functions, it will be interoperable,
given the JSON data passed around is the proper format.

Note that these `*c_void` pointers refers to a Slice pointer, containing
the offset and length of some Wasm memory, to allow for safe access between the
caller and the contract:

```rust
/// Slice refers to some heap allocated data in wasm.
/// A pointer to this can be returned over ffi boundaries.
#[repr(C)]
pub struct Slice {
    pub offset: u32,
    pub len: u32,
}
``` 
(from [memory.rs](https://github.com/confio/cosmwasm/blob/master/src/memory.rs#L7-L13))

## Implementing the Smart Contract

If you followed the [instructions above](#Creating), you should have a
runable smart contract. You may notice that all of the Wasm exports
are taken care of by `lib.rs`, which should shouldn't need to modify.
What you need to do is simply look in `contract.rs` and implement `init`
and `handle` functions, defining your custom `InitMsg` and `HandleMsg`
structs for parsing your custom message types (as json):

```rust
pub fn init<T: Storage>(store: &mut T, params: Params, msg: Vec<u8> -> 
  Result<Vec<CosmosMsg>, Error> { }

pub fn handle<T: Storage>(store: &mut T, params: Params, msg: Vec<u8> -> 
  Result<Vec<CosmosMsg>, Error> { }
```

The low-level `c_read` and `c_write` imports are nicely wrapped for you
by a `Storage` implementation (which can be swapped out between real
Wasm code and test code). This gives you a simple way to read and write
data to a custom sub-database that this contract can safely write as it wants.
It's up to you to determine which data you want to store here:

```rust
pub trait Storage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]);
}
```

## Testing the Smart Contract (rust)

For quick unit tests and useful error messages, it is often helpful to compile
the code using native build system and then test all code except for the `extern "C"`
functions (which should just be small wrappers around the real logic).

If you have non-trivial logic in the contract, please write tests using rust's
standard tooling. If you run `cargo test`, it will compile into native code
using the `debug` profile, and you get the normal test environment you know
and love. Notably, you can add plenty of requirements to `[dev-dependencies]`
in `Cargo.toml` and they will be available for your testing joy. As long
as they are only used in `#[cfg(test)]` blocks, they will never make it into
the (release) Wasm builds and have no overhead on the production artifact.

Note that for tests, you can use the `MockStorage` implementation which
gives a generic in-memory hashtable in order to quickly test your logic.
You can see a 
[simple example how to write a test](https://github.com/confio/cosmwasm/blob/81b6702d3994c8c34fb51c53176993b7e672860b/contracts/hackatom/src/contract.rs#L70-L88)
in our sample contract.
 
## Testing the Smart Contract (wasm)

You may also want to ensure the compiled contract interacts with the environment
properly. To do so, you will want to create a canonical release build of
the `<contract>.wasm` file and then write tests in with the 
same VM tooling we will use in production. This is a bit more complicated but
we added some tools to help in [cosmwasm-vm](https://github.com/confio/cosmwasm/tree/master/lib/vm)
which can be added as a `dev-dependency`.

You will need to first compile the contract using `cargo wasm`,
then load this file in the integration tests. Take a 
[look at the sample tests](https://github.com/confio/cosmwasm/blob/master/contracts/hackatom/tests/integration.rs)
to see how to do this... it is often quite easy to port a unit test
to an integration test.

## Production Builds

The above build process (`cargo wasm`) works well to produce wasm output for
testing. However, it is quite large, around 1.5 MB likely, and not suitable
for posting to the blockchain. Furthermore, it is very helpful if we have
reproducible build step so others can prove the on-chain wasm code was generated
from the published rust code.

For that, we have a separate repo, [cosmwasm-opt](https://github.com/confio/cosmwasm-opt)
that provides a [docker image](https://hub.docker.com/r/confio/cosmwasm-opt/tags)
for building. For more info, look at 
[cosmwasm-opt README](https://github.com/confio/cosmwasm-opt/blob/master/README.md#usage), 
but the quickstart guide is:

```shell script
export CODE=/path/to/your/wasm/script
docker run --rm -u $(id -u):$(id -g) -v "${CODE}":/code confio/cosmwasm-opt:1.38
```

It will output a highly size-optimized build as `contract.wasm` in `$CODE`.
With our example contract, the size went down to 126kB (from 1.6MB from `cargo wasm`).
If we didn't use serde-json, this would be much smaller still...

## Benchmarking

You may want to compare how long the contract takes to run inside the Wasm VM
compared to in native rust code, especially for computationally intensive code,
like hashing or signature verification. 

**TODO** add instructions