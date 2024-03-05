# Contract pinning

Contract pinning is a feature of the CosmWasm virtual machine which ensures that
a previously stored compiled contract code (module) is started from a dedicated
in-memory cache. Starting a module from memory takes ~45µs compared to 1.5ms
when loaded from disk (33x faster).

In contast to the node specific Least recently used (LRU) memory cache, pinning
**guarantees** this performance boost across the network. As a consequence wasmd
can charge discounted gas cost[^1].

## The caches

CosmWasm has 3 different caches for modules:

1. `FileSystemCache` the `.module` files stored in the cache directory of the
   node
2. `InMemoryCache` the LRU cache
3. `PinnedMemoryCache` a separate cache

Both memory caches (2./3.) work the same in terms of performance but their
elements are tracked separately. A pinned contract is never added to the
standard `InMemoryCache` and the size of pinned contracts is not counted towards
its cache size limit.

## Pinning and Unpinning

In order to add a contract to the `PinnedMemoryCache`, you need to call
[`Cache::pin`] in Rust or `func (vm *VM) Pin(checksum Checksum) error` in
wasmvm. To remove a contract from the cache use [`Cache::unpin`] /
`func (vm *VM) Unpin(checksum Checksum) error`. In both cases a contract is
identified by its checksum (sha256 hash of the Wasm blob).

The VM does not persist pinned memory entries. I.e. you need to call `Pin` every
time you start the process. This is implemented in [`InitializePinnedCodes` in
wasmd][initializepinnedcodes].

At the chain level pinning and unpinning is done via governance proposals. See
`MsgPinCodes`/`MsgUnpinCodes` in wasmd.

When contracts are migrated from one code to another, there is no automatic
pinning or unpinning. This is primarily since the migration of a single instance
does not means all instances of the same code become unused. In the future we
want to provide hit stats for each checksum in order to easily find unused codes
in the pinned memory cache[^2].

## Best practices

Pinning contracts is a balance between increasing memory usage and boosting
execution speed. Contracts that are known to be heavily used should be pinned.
This can includes contracts that are executed as part of begin/end block or the
IBC light client implementations of the Wasm Light Client ([08-wasm]). If a
chain is permissioned and runs on a small number of well known contracts, they
can all be pinned. A permissionless chain might select certain contracts of
strategic importance and pin them.

The estimated size of the pinned contracts is visible in the [Metrics] struct
you can access through [Prometheus](https://prometheus.io/).

## History

Pinning was developed in 2021 (CosmWasm 0.14) for the Proof of Engagement
consensus system of Tgrade which required certain contracts to be executed in
every block.

[metrics]:
  https://github.com/CosmWasm/wasmvm/blob/v2.0.0-rc.2/types/types.go#L174-L185
[`cache::pin`]:
  https://docs.rs/cosmwasm-vm/latest/cosmwasm_vm/struct.Cache.html#method.pin
[`cache::unpin`]:
  https://docs.rs/cosmwasm-vm/latest/cosmwasm_vm/struct.Cache.html#method.unpin
[08-wasm]:
  https://github.com/cosmos/ibc-go/tree/main/modules/light-clients/08-wasm
[initializepinnedcodes]:
  https://github.com/CosmWasm/wasmd/blob/v0.50.0/x/wasm/keeper/keeper.go#L1011-L1028

[^1]: https://github.com/CosmWasm/wasmd/pull/1799
[^2]: https://github.com/CosmWasm/cosmwasm/issues/2034
