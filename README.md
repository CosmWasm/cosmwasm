# CosmWasm

[![CircleCI](https://circleci.com/gh/confio/cosmwasm/tree/master.svg?style=shield)](https://circleci.com/gh/confio/cosmwasm/tree/master) 
[![Docs](https://docs.rs/cosmwasm/badge.svg)](https://docs.rs/cosmwasm)
[![crates.io](https://img.shields.io/crates/v/cosmwasm.svg)](https://crates.io/crates/cosmwasm)


**Web Assembly Smart Contracts for the Cosmos SDK**

This repo provides a useful functionality to build smart contracts that
are compatible with Cosmos SDK runtime, [currently being developed](https://github.com/cosmwasm/cosmos-sdk/issues).

## Creating a Smart Contract

You can see some examples of contracts under the `contracts` directory.
We aim to provide more tooling to help this process, but for now it is a manual step.
You can do this in the `contracts` directory if you are working in this project, or
wherever you want in your own project. 

You can follow more instructions on how to [configure a library for wasm](./Building.md)

## API entry points

Web Assembly contracts are basically black boxes. The have no default entry points,
and no access to the outside world by default. To make them useful, we need to add
a few elements. 

We explain [how to create entry points](./EntryPoints.md) in general for
rust-wasm tooling, as well as [document the required API for CosmWasm contracts](./API.md)

## Implementing the Smart Contract

**TODO** Explain what is needed

**TODO** Link to sample implementation

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

**TODO** Add some tests to sample and link them here

## Testing the Smart Contract (wasm)

You may also want to ensure the compiled contract interacts with the environment
properly. To do so, you will want to create a canonical release build of
the `<contract>.wasm` file and then write tests in go with a some tooling
in the cosmwasm/cosmos-sdk repo (**TODO**)

## Benchmarking

You may want to compare how long the contract takes to run inside the Wasm VM
compared to in native rust code, especially for computationally intensive code,
like hashing or signature verification. 

**TODO** add instructions and maybe some Dockerfile tooling