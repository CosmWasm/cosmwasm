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

To rebuild the test contract, go to the repo root and do

```sh
docker run --rm -v $(pwd):/code \
  --mount type=volume,source=$(basename $(pwd))_cache,target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  confio/cosmwasm-opt:0.6.2 ./contracts/hackatom
cp contracts/hackatom/contract.wasm lib/vm/testdata/contract_0.7.wasm
```

You must temporarily update the `[dev-dependencies]` in [Cargo.toml](../../contracts/hackatom/Cargo.toml)
to avoid compiling the singlepass backend, which only compiles with Rust nightly.
Then `cargo update --package cosmwasm-vm` to adapt `Cargo.lock`.
