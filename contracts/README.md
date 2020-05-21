# Example contracts

Those contracts are made for development purpose only. For more realistic
example contracts, see
[cosmwasm-examples](https://github.com/CosmWasm/cosmwasm-examples).

## Optimized builds

`hackatom`, `reflect` and `queue` are used for testing in other repos, e.g.
[in go-cosmwasm](https://github.com/CosmWasm/go-cosmwasm/tree/master/api/testdata).
`staking` is used by a demo project in CosmWasm JS
(https://github.com/CosmWasm/cosmwasm-js/issues/170).

To rebuild all contracts as part of a release use the following commands:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.8.0 ./contracts/hackatom

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.8.0 ./contracts/queue

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.8.0 ./contracts/reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.8.0 ./contracts/staking
```
