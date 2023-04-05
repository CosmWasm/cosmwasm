# Example contracts

Those contracts are made for development purpose only. For more realistic
example contracts, see
[cosmwasm-examples](https://github.com/CosmWasm/cosmwasm-examples).

## The contracts

Introducing the development contracts in the order they were created.

1. **hackatom** is the very first development contract that was created at a
   Cosmos Hackatom in Berlin in 2019, the event where CosmWasm was born. It is a
   very basic escrow contract. During the years of CosmWasm development, many
   more test cases were hacked into it.
2. **queue** shows and tests the newly added iterator support
   ([#181](https://github.com/CosmWasm/cosmwasm/pull/181)).
3. **reflect** is an evolution of the
   [mask contract](https://medium.com/cosmwasm/introducing-the-mask-41d11e51bccf),
   which allows the user to send messages to the contract which are then emitted
   with the contract as the sender. It later got support to handle sub messages
   and replys ([#796](https://github.com/CosmWasm/cosmwasm/pull/796)).
4. **staking** is a staking derivates example showing how the contract itself
   can be a delegator.
5. **burner** shows how contract migrations work, which were added in CosmWasm
   0.9 ([#413](https://github.com/CosmWasm/cosmwasm/pull/413)). It shuts down
   the contract my clearing all state and sending all tokens to a given address.
6. **ibc-reflect**/**ibc-reflect-send** are inspired by the idea of Interchain
   Accounts and demonstrate the power of contract to contract IBC.
   ibc-reflect-send receives a message on chain A and sends it to an ibc-reflect
   instance on chain B where the message is executed.
7. **crypto-verify** shows how to use the CosmWasm crypto APIs for signature
   verification ([#783](https://github.com/CosmWasm/cosmwasm/pull/783)).
8. **floaty** emits float operations when compiled to Wasm and allows us to test
   how tooling and the runtime deal with those operations
   ([#970](https://github.com/CosmWasm/cosmwasm/pull/970)).
9. **cyberpunk** is an attempt to cleanup hackatom and make writing runtime
   tests (cosmwasm-vm/wamsmvm) easier by avoid the need for the escrow setup
   that hackatom has.
10. **virus** is a contract that reproduces itself and does nothing useful
    beyond that, showing how to use instantiate2 from a contract.

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
  cosmwasm/rust-optimizer:0.12.13 ./contracts/burner

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_crypto_verify",target=/code/contracts/crypto-verify/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/crypto-verify

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_floaty",target=/code/contracts/floaty/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/floaty

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/code/contracts/hackatom/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/hackatom

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect",target=/code/contracts/ibc-reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/ibc-reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect_send",target=/code/contracts/ibc-reflect-send/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/ibc-reflect-send

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_queue",target=/code/contracts/queue/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/queue

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_reflect",target=/code/contracts/reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_staking",target=/code/contracts/staking/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/staking

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_virus",target=/code/contracts/virus/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13 ./contracts/virus
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
| virus       | no          | no            |
