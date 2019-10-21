# Cosmwasm VM

This is an abstraction layer around the wasmer VM to expose just what
we need to run cosmwasm contracts in a high-level manner.
This is intended both for efficient writing of unit tests, as well as a 
public API to run contracts in eg. go-cosmwasm. As such it includes all
glue code needed for typical actions, like fs caching.

## Setup

There is a demo file in `testdata/contract.wasm` - this is a compiled and
optimized version of [contracts/hackatom](https://github.com/confio/cosmwasm/tree/master/contracts/hackatom)
run through [cosmwasm-opt](https://github.com/confio/cosmwasm-opt).

