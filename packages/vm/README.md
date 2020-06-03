# CosmWasm VM

[![cosmwasm-vm on crates.io](https://img.shields.io/crates/v/cosmwasm-vm.svg)](https://crates.io/crates/cosmwasm-vm)

This is an abstraction layer around the wasmer VM to expose just what we need to
run cosmwasm contracts in a high-level manner. This is intended both for
efficient writing of unit tests, as well as a public API to run contracts in eg.
go-cosmwasm. As such it includes all glue code needed for typical actions, like
fs caching.

## Setup

There are demo files in `testdata/*.wasm`. Those are compiled and optimized
versions of
[contracts/hackatom](https://github.com/CosmWasm/cosmwasm/tree/master/contracts/hackatom)
and
[contracts/staking](https://github.com/CosmWasm/cosmwasm/tree/master/contracts/staking)
run through [rust-optimizer](https://github.com/CosmWasm/rust-optimizer).

To rebuild the test contracts, go to the repo root and do

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/code/contracts/hackatom/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.8.0 ./contracts/hackatom \
  && cp contracts/hackatom/contract.wasm packages/vm/testdata/contract_0.9.wasm
```

## Testing

By default, this repository is built and tested with the singlepass backend.
This requires running Rust nighty:

```sh
cd packages/vm
cargo +nightly test
```

To test with Rust stable, you need to switch to cranelift:

```sh
cd packages/vm
cargo test --no-default-features --features default-cranelift
```

## License

This package is part of the cosmwasm repository, licensed under the Apache
License 2.0 (see
[NOTICE](https://github.com/CosmWasm/cosmwasm/blob/master/NOTICE) and
[LICENSE](https://github.com/CosmWasm/cosmwasm/blob/master/LICENSE)).
