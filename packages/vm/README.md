# CosmWasm VM

[![cosmwasm-vm on crates.io](https://img.shields.io/crates/v/cosmwasm-vm.svg)](https://crates.io/crates/cosmwasm-vm)

This is an abstraction layer around the wasmer VM to expose just what we need to
run cosmwasm contracts in a high-level manner. This is intended both for
efficient writing of unit tests, as well as a public API to run contracts in eg.
[wasmvm](https://github.com/CosmWasm/wasmvm). As such it includes all glue code
needed for typical actions, like fs caching.

## Compatibility

A VM can support one or more contract-VM interface versions. The interface
version is communicated by the contract via a Wasm import. This is the current
compatibility list:

| cosmwasm-vm | Supported interface versions | cosmwasm-std |
| ----------- | ---------------------------- | ------------ |
| 1.0         | `interface_version_8`        | 1.0          |
| 0.16        | `interface_version_7`        | 0.16         |
| 0.15        | `interface_version_6`        | 0.15         |
| 0.14        | `interface_version_5`        | 0.14         |
| 0.13        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.12        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.11        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.10        | `cosmwasm_vm_version_3`      | 0.10         |
| 0.9         | `cosmwasm_vm_version_2`      | 0.9          |
| 0.8         | `cosmwasm_vm_version_1`      | 0.8          |

### Changes between interface versions

**interface_version_5 -> interface_version_6**

- Rename the fields from `send` to `funds` in `WasmMsg::Instantiate` and
  `WasmMsg::Execute`.
- Merge messages and sub-messages.
- Change JSON representation of IBC acknowledgements ([#975]).

[#975]: https://github.com/CosmWasm/cosmwasm/pull/975

## Setup

There are demo files in `testdata/*.wasm`. Those are compiled and optimized
versions of
[contracts/\*](https://github.com/CosmWasm/cosmwasm/tree/main/contracts/) run
through [cosmwasm/optimizer](https://github.com/CosmWasm/optimizer).

To rebuild the test contracts, go to the repo root and do

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_cyberpunk",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.1 ./contracts/cyberpunk \
  && cp artifacts/cyberpunk.wasm packages/vm/testdata/cyberpunk.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.1 ./contracts/hackatom \
  && cp artifacts/hackatom.wasm packages/vm/testdata/hackatom.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.1 ./contracts/ibc-reflect \
  && cp artifacts/ibc_reflect.wasm packages/vm/testdata/ibc_reflect.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_empty",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.1 ./contracts/empty \
  && cp artifacts/empty.wasm packages/vm/testdata/empty.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_callback",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.1 ./contracts/ibc-callbacks \
  && cp artifacts/ibc_callbacks.wasm packages/vm/testdata/ibc_callbacks.wasm
```

The `cyberpunk_rust170.wasm` for
https://github.com/CosmWasm/cosmwasm/issues/1727 is built as follows
(non-reproducible):

```sh
cd contracts/cyberpunk
rm -r target
RUSTFLAGS='-C link-arg=-s' cargo build --release --lib --target wasm32-unknown-unknown --locked
cp target/wasm32-unknown-unknown/release/cyberpunk.wasm ../../packages/vm/testdata/cyberpunk_rust170.wasm
```

The `floaty_2.0.wasm` is built using Rust nightly as follows (non-reproducible):

```sh
cd contracts/floaty
RUSTFLAGS="-C link-arg=-s -C target-feature=+nontrapping-fptoint" cargo wasm
cp target/wasm32-unknown-unknown/release/floaty.wasm ../../packages/vm/testdata/floaty_2.0.wasm
```

## Testing

By default, this repository is built and tested with the singlepass backend.

```sh
cd packages/vm
cargo test --features iterator
```

## Benchmarking

```
cd packages/vm
cargo bench --no-default-features
```

## Tools

`module_size` and `module_size.sh`

Memory profiling of compiled modules. `module_size.sh` executes `module_size`,
and uses valgrind's memory profiling tool (massif) to compute the amount of heap
memory used by a compiled module.

```
cd packages/vm
RUSTFLAGS="-g" cargo build --release --example module_size
./examples/module_size.sh ./testdata/hackatom.wasm
```

## License

This package is part of the cosmwasm repository, licensed under the Apache
License 2.0 (see [NOTICE](https://github.com/CosmWasm/cosmwasm/blob/main/NOTICE)
and [LICENSE](https://github.com/CosmWasm/cosmwasm/blob/main/LICENSE)).
