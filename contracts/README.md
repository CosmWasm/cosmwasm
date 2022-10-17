# Example contracts

Those contracts are made for development purpose only. For more realistic
example contracts, see
[cosmwasm-examples](https://github.com/CosmWasm/cosmwasm-examples).

## Optimized builds

Those development contracts are used for testing in other repos, e.g. in
[wasmvm](https://github.com/CosmWasm/wasmvm/tree/master/api/testdata) or
[cosmjs](https://github.com/cosmos/cosmjs/tree/main/scripts/wasmd/contracts).

They are [built and deployed](https://github.com/CosmWasm/cosmwasm/releases) by
the CI for every release tag. In case you need to build them manually for some
reason, use the following commands:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_burner",target=/code/contracts/burner/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/burner

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_crypto_verify",target=/code/contracts/crypto-verify/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/crypto-verify

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_floaty",target=/code/contracts/floaty/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/floaty

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/code/contracts/hackatom/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/hackatom

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect",target=/code/contracts/ibc-reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/ibc-reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect_send",target=/code/contracts/ibc-reflect-send/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/ibc-reflect-send

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_queue",target=/code/contracts/queue/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/queue

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_reflect",target=/code/contracts/reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_staking",target=/code/contracts/staking/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/staking
```

## Entry points

The development contracts in this folder contain a variety of different entry
points in order to demonstrate and test the flexibility we have.

| Contract    | Has `query` | Has `migrate` |
| ----------- | ----------- | ------------- |
| burner      | no          | yes           |
| hackatom    | yes         | yes           |
| ibc-reflect | yes         | no            |
| queue       | yes         | yes           |
| reflect     | yes         | no            |
| staking     | yes         | no            |
