# cosmwasm-check

It allows checking if the Wasm binary is a proper smart contract that's ready to
be uploaded to the blockchain.

## Installation

```sh
cargo install cosmwasm-check
```

## Usage

Get help and info:

```sh
cosmwasm-check -h
```

Check some contracts:

```sh
cosmwasm-check artifacts/hackatom.wasm artifacts/burner.wasm
```

Check an entire directory of contracts (shell dependent):

```sh
cosmwasm-check artifacts/*.wasm
```

Check if a contract would ran on a blockchain with a specific set of
capabilities:

```sh
cosmwasm-check --available-capabilities iterator,osmosis,friendship artifacts/hackatom.wasm
```

## License

This package is part of the cosmwasm repository, licensed under the Apache
License 2.0 (see [NOTICE](https://github.com/CosmWasm/cosmwasm/blob/main/NOTICE)
and [LICENSE](https://github.com/CosmWasm/cosmwasm/blob/main/LICENSE)).
