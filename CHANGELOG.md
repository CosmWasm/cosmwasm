# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased]

## [3.0.1] - 2025-06-26

## Added

- cosmwasm-std: Add missing export for `ValidatorMetadata` ([#2511])

[#2511]: https://github.com/CosmWasm/cosmwasm/pull/2511

## [3.0.0] - 2025-06-23

## Added

- cosmwasm-std: Implement `From<Uint64> for u{64,128}`,
  `From<Uint128> for u128`, `From<Int64> for i{64,128}`, and
  `From<Int128> for i128` ([#2268])
- cosmwasm-std: Implement `Uint128::from_{be,le}_bytes` and
  `Uint64::from_{be,le}_bytes`. ([#2269])
- cosmwasm-std: Added new `Ibc2Msg` and `CosmosMsg::Ibc2` variant ([#2340],
  [#2390], [#2403])
- cosmwasm-std: Added `ibc2_port` to `ContractInfoResponse`. ([#2390], [#2403])
- cosmwasm-vm: Added `ibc2_packet_receive` entrypoint ([#2403])
- cosmwasm-vm: Add IBC Callbacks entrypoints to the `Entrypoints` enum.
  ([#2438])
- cosmwasm-std: Add `WriteAcknowledgement` to `Ibc2Msg` - ([#2425])
- cosmwasm-vm: Add `ibc2_packet_timeout` entrypoint - ([#2454])
- cosmwasm-std: Add `Ibc2PacketTimeoutMsg` message - ([#2454])
- cosmwasm-std: Add `cosmwasm_std::testing::Envs`, which is an `Env` factory for
  testing environments. It auto-increments block heights and timestamps. It
  allows for advanced configurations such as custom address prefixes. ([#2442])
- cosmwasm-vm: Add support for `ibc2_packet_ack` endpoint ([#2474])
- cosmwasm-std: Add support for `ibc2_packet_ack` endpoint ([#2474])
- cosmwasm-std: Add `Ibc2PacketSendMsg` message - ([#2477])
- cosmwasm-vm: Add `ibc2_packet_send` entrypoint ([#2477])
- cosmwasm-std: Add Tx hash to TransactionInfo and make it non exhaustive
  ([#2480])
- cosmwasm-std: Add `WasmQuery::RawRange` query to allow querying raw storage
  ranges of different contracts. ([#2471])
- cosmwasm-std: Add `transfer` field to `IbcDestinationCallbackMsg`, providing
  an easier way to handle an IBC transfer in a destination callback. ([#2484])
- cw-schema/cw-schema-derive: Add new easily compressible schema and associated
  proc-macro ([#2495])
- cosmwasm-schema: Generate new cw-schemas alongside JSON schemas ([#2495])

## Changed

- cosmwasm-std: Deprecate `abort` feature. The panic handler is now always
  enabled. ([#2337])
- cosmwasm-std: Document safety invariants of the internal memory repr ([#2344])
- cosmwasm-std: Enforce non-null pointers using `ptr::NonNull` in the internal
  memory repr ([#2344])
- cosmwasm-std: Make `instantiate2_address_impl` public and let it take a new
  `len` argument to allow truncating address data as part of the generation
  process. ([#2155])
- cosmwasm-vm: Updated wasmer to 5.0.4 ([#2374])
- cosmwasm-vm: Charge gas for `write_region` ([#2378])
- cosmwasm-vm: Remove the `cranelift` feature. This was doing nothing since
  2.2.0 already. ([#2262])
- cosmwasm-std: Remove previously deprecated `from_slice`, `from_binary`,
  `to_vec` and `to_binary`. ([#2156])
- cosmwasm-vm: The testing functions `cosmwasm_vm::testing::*` do not require
  the contract's message types to implement `schemars::JsonSchema` anymore. This
  makes the use of `schemars` optional for contracts. ([#2201])
- cosmwasm-std: Remove `schemars::JsonSchema` requirement from `CustomMsg`.
  ([#2201])
- cosmwasm-std: `Int256::new`/`Int512::new` now take an `i128` argument instead
  of bytes. Use `::from_be_bytes` if you need the old behaviour. ([#2367])
- cosmwasm-std: `Uint256::new`/`Uint512::new` now take an `u128` argument
  instead of bytes. Use `::from_be_bytes` if you need the old behaviour.
  ([#2367])
- cosmwasm-std: Deprecate `{Decimal,Decimal256}::raw` and
  `{SignedDecimal,SignedDecimal256}::raw` in favour of e.g.
  `Decimal::new(Uint128::new(value))`. ([#2399])
- cosmwasm-std: Deprecate `Uint256::from_u128` and `Int256::from_i128` in favour
  of `::new`. ([#2399])
- cosmwasm-std: Move `MemoryStorage` to `cosmwasm_std::testing::MockStorage`.
  ([#2237])
- cosmwasm-std: Remove previously deprecated `cosmwast_std::testing::mock_info`.
  Use `cosmwasm_std::testing::message_info` instead. ([#2393])
- cosmwasm-std: Remove abort feature. ([#2141])
- cosmwasm-std: Change `Coin::amount` to `Uint256` instead of `Uint128`.
  ([#2458])
- cosmwasm-std: Replace dependency `serde-json-wasm` with `serde_json`.
  ([#2195])
- cosmwasm-std: Make `GovMsg` `#[non_exhaustive]` for consistency. ([#2347])
- cosmwasm-crypto: Upgrade ark-\* dependencies to 0.5.0. ([#2432])
- cosmwasm-std: Remove support for `BankQuery::AllBalances` and
  `query_all_balances`. ([#2433])
- cosmwasm-std: source_client instead of channel_id in IBCv2 - ([#2450])
- cosmwasm-std: Remove previously deprecated `IbcQuery::ListChannels` and
  `ListChannelsResponse`. ([#2223])
- cosmwasm-std: Remove export of `ExternalApi`, `ExternalQuerier` and
  `ExternalStorage` as those are only needed by export implementations in
  cosmwasm-std. ([#2467])
- cosmwasm-vm: Update wasmer to 5.0.6. ([#2472])
- cosmwasm-std: Add a new `exports` feature which needs to be enabled for the
  primary cosmwasm_std dependency of a contract.
- cosmwasm-vm: Enable partial reference-type support, enabling contracts
  compiled with Rust 1.82 or newer to be stored. ([#2473])
- cosmwasm-std: Removed IBC fees ([#2479])
- cosmwasm-schema: Remove unused result types from trait definition (#[2495])
- cosmwasm-std: Split up `Validator` type into `Validator` and
  `ValidatorMetadata` to allow adding more fields to `ValidatorResponse` in the
  future. ([#2501])
- cosmwasm-std: Redesigned `StdError` to be more flexible and less immutable
  ([#2500])

## Fixed

- cosmwasm-schema: The schema export now doesn't overwrite existing
  `additionalProperties` values anymore ([#2310])
- cosmwasm-vm: Fix CWA-2025-002.
- cosmwasm-std: Fix deserialization of `DenomMetadata`. ([#2417])

[#2141]: https://github.com/CosmWasm/cosmwasm/issues/2141
[#2155]: https://github.com/CosmWasm/cosmwasm/issues/2155
[#2156]: https://github.com/CosmWasm/cosmwasm/issues/2156
[#2195]: https://github.com/CosmWasm/cosmwasm/issues/2195
[#2201]: https://github.com/CosmWasm/cosmwasm/issues/2201
[#2223]: https://github.com/CosmWasm/cosmwasm/issues/2223
[#2237]: https://github.com/CosmWasm/cosmwasm/issues/2237
[#2262]: https://github.com/CosmWasm/cosmwasm/issues/2262
[#2268]: https://github.com/CosmWasm/cosmwasm/issues/2268
[#2269]: https://github.com/CosmWasm/cosmwasm/issues/2269
[#2310]: https://github.com/CosmWasm/cosmwasm/pull/2310
[#2337]: https://github.com/CosmWasm/cosmwasm/issues/2337
[#2340]: https://github.com/CosmWasm/cosmwasm/pull/2340
[#2344]: https://github.com/CosmWasm/cosmwasm/pull/2344
[#2347]: https://github.com/CosmWasm/cosmwasm/issues/2347
[#2367]: https://github.com/CosmWasm/cosmwasm/issues/2367
[#2374]: https://github.com/CosmWasm/cosmwasm/issues/2374
[#2378]: https://github.com/CosmWasm/cosmwasm/issues/2378
[#2390]: https://github.com/CosmWasm/cosmwasm/issues/2390
[#2393]: https://github.com/CosmWasm/cosmwasm/issues/2393
[#2399]: https://github.com/CosmWasm/cosmwasm/pull/2399
[#2403]: https://github.com/CosmWasm/cosmwasm/pull/2403
[#2417]: https://github.com/CosmWasm/cosmwasm/pull/2417
[#2425]: https://github.com/CosmWasm/cosmwasm/pull/2425
[#2432]: https://github.com/CosmWasm/cosmwasm/pull/2432
[#2433]: https://github.com/CosmWasm/cosmwasm/pull/2433
[#2438]: https://github.com/CosmWasm/cosmwasm/pull/2438
[#2442]: https://github.com/CosmWasm/cosmwasm/pull/2442
[#2450]: https://github.com/CosmWasm/cosmwasm/pull/2450
[#2454]: https://github.com/CosmWasm/cosmwasm/pull/2454
[#2458]: https://github.com/CosmWasm/cosmwasm/pull/2458
[#2467]: https://github.com/CosmWasm/cosmwasm/pull/2467
[#2471]: https://github.com/CosmWasm/cosmwasm/pull/2471
[#2472]: https://github.com/CosmWasm/cosmwasm/pull/2472
[#2473]: https://github.com/CosmWasm/cosmwasm/pull/2473
[#2477]: https://github.com/CosmWasm/cosmwasm/pull/2477
[#2479]: https://github.com/CosmWasm/cosmwasm/pull/2479
[#2480]: https://github.com/CosmWasm/cosmwasm/pull/2480
[#2484]: https://github.com/CosmWasm/cosmwasm/pull/2484
[#2495]: https://github.com/CosmWasm/cosmwasm/pull/2495
[#2500]: https://github.com/CosmWasm/cosmwasm/pull/2500
[#2501]: https://github.com/CosmWasm/cosmwasm/pull/2501

## [2.2.0] - 2024-12-17

### Added

- cosmwasm-std: Add `from_msgpack`, `to_msgpack_vec` and `to_msgpack_binary`.
  These functions are meant to be used similarly to their JSON counterparts.
  [MessagePack](https://msgpack.org) is a more compact, binary encoding.
  ([#2118])
- cosmwasm-std: Add `IbcMsg::{PayPacketFee, PayPacketFeeAsync}` and
  `IbcQuery::FeeEnabledChannel` to allow contracts to incentivize IBC packets
  using IBC Fees. ([#2196])
- cosmwasm-vm: Add `Config` that allows to configure the limits for static Wasm
  validation. ([#2220])
- cosmwasm-check: Add `--wasm-limits` flag to supply configured limits for
  static validation. ([#2220])
- cosmwasm-std: Add `migrate_with_info` call implementation for the extended
  `migrate` entrypoint function ([#2212])
- cosmwasm-vm: Export a new `migrate_with_info` function ([#2212])
- cosmwasm-derive: Add support for migrate method with
  `migrate_info: MigrateInfo` argument. ([#2212])
- cosmwasm-vm: Add `Cache::store_code`

[#2118]: https://github.com/CosmWasm/cosmwasm/pull/2118
[#2196]: https://github.com/CosmWasm/cosmwasm/pull/2196
[#2220]: https://github.com/CosmWasm/cosmwasm/pull/2220
[#2212]: https://github.com/CosmWasm/cosmwasm/pull/2212

### Changed

- cosmwasm-std: `Binary`, `HexBinary` and `Checksum` are now encoded as binary
  blobs when used together with a "compact" `serde` encoding. A compact encoding
  is an encoding that returns `false` from
  [`is_human_readable`](https://docs.rs/serde/latest/serde/trait.Serializer.html#method.is_human_readable).
  This is to make these types more efficient when used together with the new
  [MessagePack](https://msgpack.org) encoding. ([#2118])
- cosmwasm-std: Let `mock_env` return a contract address that is valid bech32
  and uses the same bech32 prefix as `MockApi`; Change `MOCK_CONTRACT_ADDR`
  value to match the contract address from `mock_env`. ([#2211])
- cosmwasm-vm: Let `mock_env` return a contract address that is valid bech32 and
  uses the same bech32 prefix as `MockApi`; Change `MOCK_CONTRACT_ADDR` value to
  match the contract address from `mock_env`. ([#2211])
- cosmwasm-derive: Automatically detect whether the package is a dependency or
  the primary package, only expanding entrypoints for the primary package. This
  effectively deprecates the usage of the `library` feature pattern.

  Note: This feature does **NOT** interact well with workspaces due to a cargo
  bug. If you have multiple contracts in a workspace, you might still want to
  use the library feature ([#2246])

- cosmwasm-std: Deprecate `BankQuery::AllBalances` and `IbcQuery::ListChannels`.
  Both are inherently problematic to use because the returned entries are
  unbounded. ([#2247])
- cosmwasm-vm: Upgrade Wasmer to 4.3.7; Bump `MODULE_SERIALIZATION_VERSION` to
  "v20". ([#2255])
- cosmwasm-vm: Effectively remove the `cranelift` feature. It still exists but
  is only a no-op for semver compatibility. It will now unconditionally use the
  singlepass compiler. ([#2260])
- cosmwasm-vm: Automatically derive cache version from function hashes and the
  Wasmer version ([#2250])

[#2118]: https://github.com/CosmWasm/cosmwasm/pull/2118
[#2211]: https://github.com/CosmWasm/cosmwasm/issues/2211
[#2246]: https://github.com/CosmWasm/cosmwasm/pull/2246
[#2247]: https://github.com/CosmWasm/cosmwasm/pull/2247
[#2255]: https://github.com/CosmWasm/cosmwasm/pull/2255
[#2260]: https://github.com/CosmWasm/cosmwasm/pull/2260
[#2250]: https://github.com/CosmWasm/cosmwasm/pull/2250

## [2.1.5] - 2024-12-10

- cosmwasm-vm: Add `Cache::store_code`

## [2.1.4] - 2024-09-23

### Fixed

- cosmwasm-vm: Fix CWA-2024-007 and CWA-2024-008.

## [2.1.3] - 2024-08-08

### Fixed

- cosmwasm-vm: Problem with caching related to CWA-2024-004. Please upgrade
  directly to this version instead of the previous one.

## [2.1.2] - 2024-08-08

### Fixed

- cosmwasm-vm: Fix CWA-2024-004

## [2.1.1] - 2024-07-30

### Fixed

- cosmwasm-std: Make fields of `IbcAckCallbackMsg` and `IbcTimeoutCallbackMsg`
  public. ([#2191])
- cosmwasm-std: Add default implementation for `Storage::range` to make
  `iterator` feature additive. ([#2197])

[#2191]: https://github.com/CosmWasm/cosmwasm/pull/2191
[#2197]: https://github.com/CosmWasm/cosmwasm/pull/2197

## [2.1.0] - 2024-07-11

### Fixed

- cosmwasm-std: Fix CWA-2024-002
- cosmwasm-std: Fix `Reply` deserialization on CosmWasm 1.x chains ([#2159])
- cosmwasm-std: Updated `QueryRequest` enum to use the default generic parameter
  `Empty`. ([#2165])

[#2159]: https://github.com/CosmWasm/cosmwasm/pull/2159
[#2165]: https://github.com/CosmWasm/cosmwasm/pull/2165

### Added

- cosmwasm-vm: Add `secp256r1_verify` and `secp256r1_recover_pubkey` imports for
  ECDSA signature verification over secp256r1. ([#1983], [#2057], [#2058])
- cosmwasm-vm: Add metrics for the pinned memory cache ([#2059])
- cosmwasm-derive: The crate used in the expansion can now be renamed ([#2068])
- cosmwasm-schema-derive: The crate used in the expansion can now be renamed
  ([#2070])
- cosmwasm-std: The decimal types now implement `TryFrom` for their respective
  integer representations ([#2075])
- cosmwasm-std: Implement `&T + T` and `&T op &T` for `Uint64`, `Uint128`,
  `Uint256` and `Uint512`; improve panic message for `Uint64::add` and
  `Uint512::add` ([#2092])
- cosmwasm-std: Add `{CosmosMsg,SubMsg,Response}::change_custom` to change the
  custom message type ([#2099])
- cosmwasm-std: Add `Uint{64,128,256,512}::strict_add` and `::strict_sub` which
  are like the `Add`/`Sub` implementations but `const`. ([#2098], [#2107])
- cosmwasm-std: Let `Timestamp::plus_nanos`/`::minus_nanos` use
  `Uint64::strict_add`/`::strict_sub` and document overflows. ([#2098], [#2107])
- cosmwasm-std: Add `QuerierWrapper::query_grpc` helper for gRPC queries.
  ([#2120])
- cosmwasm-derive: Add `migrate_version` attribute for `migrate` entrypoints
  ([#2124], [#2166])
- cosmwasm-vm: Read the migrate version from Wasm modules and return them as
  part of `AnalyzeReport` ([#2129], [#2166])
- cosmwasm-vm: Add `bls12_381_aggregate_g1`, `bls12_381_aggregate_g2`,
  `bls12_381_pairing_equality`, `bls12_381_hash_to_g1`, and
  `bls12_381_hash_to_g2` to enable BLS12-381 curve operations, such as verifying
  pairing equalities ([#2106])
- cosmwasm-std: Add IBC Callbacks support, including two new entrypoints
  `ibc_source_callback` and `ibc_destination_callback`, as well as the
  `IbcCallbackRequest` type. ([#2025])
- cosmwasm-vm: Add support for the two new IBC Callbacks entrypoints. ([#2025])
- cosmwasm-std: Add `TransferMsgBuilder` to more easily create an
  `IbcMsg::Transfer` with different kinds of memo values, including IBC
  Callbacks memo values. ([#2167])
- cosmwasm-std: Add `IbcMsg::WriteAcknowledgement` for async IBC
  acknowledgements ([#2130])
- cosmwasm-std: Add derive attributes for `Order` ([#2174])

[#1983]: https://github.com/CosmWasm/cosmwasm/pull/1983
[#2025]: https://github.com/CosmWasm/cosmwasm/pull/2025
[#2057]: https://github.com/CosmWasm/cosmwasm/pull/2057
[#2058]: https://github.com/CosmWasm/cosmwasm/pull/2058
[#2068]: https://github.com/CosmWasm/cosmwasm/pull/2068
[#2075]: https://github.com/CosmWasm/cosmwasm/pull/2075
[#2092]: https://github.com/CosmWasm/cosmwasm/pull/2092
[#2098]: https://github.com/CosmWasm/cosmwasm/pull/2098
[#2099]: https://github.com/CosmWasm/cosmwasm/pull/2099
[#2106]: https://github.com/CosmWasm/cosmwasm/pull/2106
[#2107]: https://github.com/CosmWasm/cosmwasm/pull/2107
[#2120]: https://github.com/CosmWasm/cosmwasm/pull/2120
[#2124]: https://github.com/CosmWasm/cosmwasm/pull/2124
[#2129]: https://github.com/CosmWasm/cosmwasm/pull/2129
[#2130]: https://github.com/CosmWasm/cosmwasm/pull/2130
[#2166]: https://github.com/CosmWasm/cosmwasm/pull/2166
[#2167]: https://github.com/CosmWasm/cosmwasm/pull/2167
[#2174]: https://github.com/CosmWasm/cosmwasm/pull/2174

### Changed

- cosmwasm-std: Enable `add_event` and `add_events` functions to process types
  implementing `Into<Event>` ([#2044])
- cosmwasm-vm: Improve performance of the `Cache::analyze` function ([#2051])
- cosmwasm-derive: Update to `syn` v2 ([#2063])
- cosmwasm-schema-derive: Update to `syn` v2 ([#2063])
- cosmwasm-schema-derive: Improve emitted error messages ([#2063])
- cosmwasm-schema: `#[cw_serde]` now doesn't add `#[serde(deny_unknown_fields)]`
  to the expanded code anymore ([#2080])
- cosmwasm-std: Improve performance of `Uint{64,128,256,512}::isqrt` ([#2108])
- cosmwasm-std: Deprecate "compact" serialization of `Binary`, `HexBinary`,
  `Checksum` ([#2125])
- cosmwasm-vm: Update wasmer to 4.3.3 ([#2147], [#2153], [#2182])
- cosmwasm-vm: Rebalance gas costs for cryptographic functions and wasm
  instructions. ([#2152])
- cosmwasm-std: Add message_info and deprecate mock_info ([#2160])

[#2044]: https://github.com/CosmWasm/cosmwasm/pull/2044
[#2051]: https://github.com/CosmWasm/cosmwasm/pull/2051
[#2059]: https://github.com/CosmWasm/cosmwasm/pull/2059
[#2063]: https://github.com/CosmWasm/cosmwasm/pull/2063
[#2070]: https://github.com/CosmWasm/cosmwasm/pull/2070
[#2080]: https://github.com/CosmWasm/cosmwasm/pull/2080
[#2108]: https://github.com/CosmWasm/cosmwasm/pull/2108
[#2125]: https://github.com/CosmWasm/cosmwasm/pull/2125
[#2147]: https://github.com/CosmWasm/cosmwasm/pull/2147
[#2152]: https://github.com/CosmWasm/cosmwasm/pull/2152
[#2153]: https://github.com/CosmWasm/cosmwasm/pull/2153
[#2160]: https://github.com/CosmWasm/cosmwasm/pull/2160
[#2182]: https://github.com/CosmWasm/cosmwasm/pull/2182

## [2.0.8] - 2024-12-10

- cosmwasm-vm: Add `Cache::store_code`

## [2.0.7] - 2024-09-23

### Fixed

- cosmwasm-vm: Fix CWA-2024-007 and CWA-2024-008.

## [2.0.6] - 2024-08-08

### Fixed

- cosmwasm-vm: Problem with caching related to CWA-2024-004. Please upgrade
  directly to this version instead of the previous one.

## [2.0.5] - 2024-08-08

### Fixed

- cosmwasm-vm: Fix CWA-2024-004

## [2.0.4] - 2024-06-03

### Fixed

- cosmwasm-std: Fix `Reply` deserialization on CosmWasm 1.x chains ([#2158])

[#2158]: https://github.com/CosmWasm/cosmwasm/pull/2158

### Changed

- cosmwasm-std: Add message_info and deprecate mock_info ([#2160])

[#2160]: https://github.com/CosmWasm/cosmwasm/pull/2160

## [2.0.3] - 2024-05-10

### Changed

- cosmwasm-std: Deprecate "compact" serialization of `Binary`, `HexBinary`,
  `Checksum` ([#2128])

[#2128]: https://github.com/CosmWasm/cosmwasm/pull/2128

## [2.0.2] - 2024-04-24

### Fixed

- cosmwasm-std: Fix CWA-2024-002

### Added

- cosmwasm-std: Implement `&T + T` and `&T op &T` for `Uint64`, `Uint128`,
  `Uint256` and `Uint512`; improve panic message for `Uint64::add` and
  `Uint512::add` ([#2092])
- cosmwasm-std: Add `Uint{64,128,256,512}::strict_add` and `::strict_sub` which
  are like the `Add`/`Sub` implementations but `const`. ([#2098], [#2107])

[#2092]: https://github.com/CosmWasm/cosmwasm/pull/2092
[#2098]: https://github.com/CosmWasm/cosmwasm/pull/2098
[#2107]: https://github.com/CosmWasm/cosmwasm/pull/2107

### Changed

- cosmwasm-std: Let `Timestamp::plus_nanos`/`::minus_nanos` use
  `Uint64::strict_add`/`::strict_sub` and document overflows. ([#2098], [#2107])

[#2098]: https://github.com/CosmWasm/cosmwasm/pull/2098
[#2107]: https://github.com/CosmWasm/cosmwasm/pull/2107

## [2.0.1] - 2024-04-03

### Fixed

- cosmwasm-std: Correctly deallocate vectors that were turned into a `Region`
  via `release_buffer` ([#2062])
- cosmwasm-std: Add back `CosmosMsg::Stargate` case to support new contracts on
  chains with older CosmWasm versions. ([#2083])

[#2062]: https://github.com/CosmWasm/cosmwasm/pull/2062
[#2083]: https://github.com/CosmWasm/cosmwasm/pull/2083

## [2.0.0] - 2024-03-12

### Fixed

- cosmwasm-vm: Fix memory increase issue (1.3 -> 1.4 regression) by avoiding the
  use of a long running Wasmer Engine. ([#1978])
- cosmwasm-vm: Fix CWA-2023-004. ([#1996])

[#1978]: https://github.com/CosmWasm/cosmwasm/issues/1978
[#1996]: https://github.com/CosmWasm/cosmwasm/issues/1996

### Added

- cosmwasm-std: Add `SubMsg:reply_never` constructor ([#1929])
- cosmwasm-std: Add optional memo field to `IbcMsg::Transfer`. ([#1878])
- cosmwasm-std: Add `Reply::gas_used`. ([#1954])
- cosmwasm-std: Add `SubMsgResponse::msg_responses` and deprecate
  `SubMsgResponse::data`. Add new type `MsgResponse`. ([#1903])
- cosmwasm-std: Add `cosmwasm_2_0` feature to enable 2.0 specific functionality.
  ([#1974])
- cosmwasm-std: Add new field `payload` to `SubMsg` and `Reply`. This is binary
  data the contract can set in a contract specific format and get back then the
  `reply` entry point is called. `SubMsg::with_payload` allows setting the
  payload on an existing `SubMsg`. ([#2008])

[#1878]: https://github.com/CosmWasm/cosmwasm/pull/1878
[#1903]: https://github.com/CosmWasm/cosmwasm/pull/1903
[#1929]: https://github.com/CosmWasm/cosmwasm/pull/1929
[#1954]: https://github.com/CosmWasm/cosmwasm/pull/1954
[#1974]: https://github.com/CosmWasm/cosmwasm/pull/1974
[#2008]: https://github.com/CosmWasm/cosmwasm/pull/2008

### Changed

- cosmwasm-std: Replace `ContractInfoResponse::new` with new (unstable)
  constructor, remove `SubMsgExecutionResponse` (Use `SubMsgResponse` instead)
  and remove `PartialEq<&str> for Addr` (validate the address and use
  `PartialEq<Addr> for Addr` instead). ([#1879])
- cosmwasm-std: `Uint{64,128}::full_mul` now take `Into<Self>` as an argument.
  ([#1874])
- cosmwasm-vm: Make `CacheOptions` non-exhaustive and add a constructor.
  ([#1898])
- cosmwasm-std: `Coin::new` now takes `Into<Uint128>` instead of `u128` as the
  first argument and `DecCoin::new` takes `Into<Decimal256>` instead of
  `Decimal256`. ([#1902])
- cosmwasm-std: Make inner values of `CanonicalAddr` and `Binary` private and
  add constructor for `Binary`. ([#1876])
- cosmwasm-vm: Make inner value of `Size` private and add constructor. ([#1876])
- cosmwasm-vm: Reduce gas values by a factor of 1000. ([#1884])
- cosmwasm-std: Upgrade to `serde-json-wasm` 1.0. This means `u128` and `i128`
  are now serialized as numbers instead of strings. Use `Uint128` and `Int128`
  instead. ([#1939])
- cosmwasm-std: Add `ack` parameter to `IbcReceiveResponse::new` and remove
  `IbcReceiveResponse::set_ack` ([#1940])
- cosmwasm-std: Make `BalanceResponse`, `AllBalanceResponse`,
  `DelegationRewardsResponse`, `DelegatorReward`, `DelegatorValidatorsResponse`,
  `PortIdResponse`, `ListChannelsResponse`, `ChannelResponse`,
  `BondedDenomResponse`, `AllDelegationsResponse`, `Delegation`,
  `DelegationResponse`, `FullDelegation`, `AllValidatorsResponse`,
  `ValidatorResponse` and `Validator` non-exhaustive. Add `Validator::create`
  and `FullDelegation::create` to allow creating them in a stable way. Use
  `Addr` type for `ContractInfoResponse::{creator, admin}`. ([#1883])
- cosmwasm-std: Change `DistributionQuerier::new` to take `IntoIterator` instead
  of `HashMap`. ([#1941])
- cosmwasm-vm: Make `instantiate` entrypoint optional. ([#1933])
- cosmwasm-std: Rename `CosmosMsg::Stargate` to `CosmosMsg::Any` and use a
  nested msg struct like in other messages. ([#1926])
- cosmwasm-vm: Add `AnalysisReport::entrypoints` and make
  `AnalysisReport::required_capabilities` a `BTreeSet`. ([#1949])
- cosmwasm-std: Add `Checksum` type and change type of
  `CodeInfoResponse::checksum` to that. ([#1944])
- cosmwasm-std: Removed `backtraces` feature, use the `RUST_BACKTRACE=1` env
  variable instead. Error variants that previously only contained a `backtrace`
  field with the feature enabled now always contain it. ([#1967])
- cosmwasm-vm: Removed `backtraces` feature, use the `RUST_BACKTRACE=1` env
  variable instead. All `VmError` variants now have a `backtrace` field.
  ([#1967])
- cosmwasm-std: Replace `MockApi` with bech32 implementation. ([#1914])
- cosmwasm-vm: Replace `MockApi` with bech32 implementation. ([#1914])
- cosmwasm-std: Make `IbcReceiveResponse::acknowledgement` optional and add
  `IbcReceiveResponse::without_ack` constructor. ([#1892])
- cosmwasm-std: Add `std` feature and make it a default feature. ([#1971])
- cosmwasm-std: Add `QueryRequest::Grpc` and deprecate `QueryRequest::Stargate`.
  ([#1973])
- cosmwasm-std: Remove `update_balance`, `set_denom_metadata`,
  `set_withdraw_address`, `set_withdraw_addresses`, `clear_withdraw_addresses`,
  `update_ibc` and `update_staking` from `MockQuerier` and expose the underlying
  queriers directly. ([#1977])
- cosmwasm-vm: Rename `BackendApi::canonical_address`/`::human_address` to
  `::addr_canonicalize`/`::addr_humanize` for consistency.
- cosmwasm-vm: Add `BackendApi::addr_validate` to avoid having to do two calls
  from Rust into Go.
- cosmwasm-vm: Upgrade Wasmer to 4.2.6; Bump `MODULE_SERIALIZATION_VERSION` to
  "v9". ([#1992], [#2042])
- cosmwasm-std: Rename `GovMsg::vote` to `GovMsg::option` ([#1999])
- cosmwasm-vm: Read `Region` from Wasm memory as bytes and convert to `Region`
  afterwards ([#2005])
- cosmwasm-vm: Limit total number of function parameters in
  `check_wasm_functions` and increase max function count and max parameter
  count. ([#1991])

[#1874]: https://github.com/CosmWasm/cosmwasm/pull/1874
[#1876]: https://github.com/CosmWasm/cosmwasm/pull/1876
[#1879]: https://github.com/CosmWasm/cosmwasm/pull/1879
[#1883]: https://github.com/CosmWasm/cosmwasm/pull/1883
[#1884]: https://github.com/CosmWasm/cosmwasm/pull/1884
[#1892]: https://github.com/CosmWasm/cosmwasm/pull/1892
[#1898]: https://github.com/CosmWasm/cosmwasm/pull/1898
[#1902]: https://github.com/CosmWasm/cosmwasm/pull/1902
[#1914]: https://github.com/CosmWasm/cosmwasm/pull/1914
[#1926]: https://github.com/CosmWasm/cosmwasm/pull/1926
[#1933]: https://github.com/CosmWasm/cosmwasm/pull/1933
[#1939]: https://github.com/CosmWasm/cosmwasm/pull/1939
[#1940]: https://github.com/CosmWasm/cosmwasm/pull/1940
[#1941]: https://github.com/CosmWasm/cosmwasm/pull/1941
[#1944]: https://github.com/CosmWasm/cosmwasm/pull/1944
[#1949]: https://github.com/CosmWasm/cosmwasm/pull/1949
[#1967]: https://github.com/CosmWasm/cosmwasm/pull/1967
[#1971]: https://github.com/CosmWasm/cosmwasm/pull/1971
[#1973]: https://github.com/CosmWasm/cosmwasm/pull/1973
[#1977]: https://github.com/CosmWasm/cosmwasm/pull/1977
[#1991]: https://github.com/CosmWasm/cosmwasm/pull/1991
[#1992]: https://github.com/CosmWasm/cosmwasm/pull/1992
[#1999]: https://github.com/CosmWasm/cosmwasm/pull/1999
[#2005]: https://github.com/CosmWasm/cosmwasm/pull/2005
[#2042]: https://github.com/CosmWasm/cosmwasm/pull/2042

### Removed

- cosmwasm-std: Remove `Mul<Decimal> for Uint128` and
  `Mul<Decimal256> for Uint256`. Use `Uint{128,256}::mul_floor` instead.
  ([#1890])
- cosmwasm-std: Remove operand strings from `OverflowError`,
  `ConversionOverflowError` and `DivideByZeroError`. ([#1896])
- cosmwasm-std: Remove old IBC version and make v3 the default. ([#1875])
- cosmwasm-storage: Removed, use [cw-storage-plus] instead. ([#1936])
- cosmwasm-std: Remove `IbcReceiveResponse`'s `Default` implementation. Use
  `IbcReceiveResponse::new` instead. ([#1942])
- cosmwasm-vm: Remove `InstanceOptions::print_debug` flag. Set your own handler
  using `Instance::set_debug_handler`. ([#1953])
- cosmwasm-vm: Remove `allow_interface_version_7` feature and all related
  functionality. ([#1952])
- cosmwasm-vm: Remove `Checksum`. Use `cosmwasm_std::Checksum` instead.
  ([#1944])

[cw-storage-plus]: https://github.com/CosmWasm/cw-storage-plus
[#1875]: https://github.com/CosmWasm/cosmwasm/pull/1875
[#1890]: https://github.com/CosmWasm/cosmwasm/pull/1890
[#1896]: https://github.com/CosmWasm/cosmwasm/pull/1896
[#1936]: https://github.com/CosmWasm/cosmwasm/pull/1936
[#1942]: https://github.com/CosmWasm/cosmwasm/pull/1942
[#1952]: https://github.com/CosmWasm/cosmwasm/pull/1952
[#1953]: https://github.com/CosmWasm/cosmwasm/pull/1953

## [1.5.9] - 2024-12-10

### Added

- cosmwasm-vm: Add `Cache::store_code`

## [1.5.8] - 2024-09-23

### Fixed

- cosmwasm-vm: Fix CWA-2024-007 and CWA-2024-008.

## [1.5.7] - 2024-08-08

### Fixed

- cosmwasm-vm: Problem with caching related to CWA-2024-004. Please upgrade
  directly to this version instead of the previous one.

## [1.5.6] - 2024-08-08

### Fixed

- cosmwasm-vm: Fix CWA-2024-004

### Changed

- cosmwasm-std: Backport PR that changed the version pinned dependency
  `k256 = { version = "=0.13.1", features = ["ecdsa"] }` to the open version
  range ^0.13.3 by avoiding a normalization of the public key in
  `secp256k1_recover_pubkey`. ([#2014], [#2198])

[#2014]: https://github.com/CosmWasm/cosmwasm/pull/2014
[#2198]: https://github.com/CosmWasm/cosmwasm/pull/2198

## [1.5.5] - 2024-05-10

### Changed

- cosmwasm-std: Deprecate "compact" serialization of `Binary`, `HexBinary`
  ([#2126])

[#2126]: https://github.com/CosmWasm/cosmwasm/pull/2126

## [1.5.4]

### Fixed

- cosmwasm-std: Fix CWA-2024-002

### Added

- cosmwasm-std: Implement `&T + T` and `&T op &T` for `Uint64`, `Uint128`,
  `Uint256` and `Uint512`; improve panic message for `Uint64::add` and
  `Uint512::add` ([#2092])
- cosmwasm-std: Add `Uint{64,128,256,512}::strict_add` and `::strict_sub` which
  are like the `Add`/`Sub` implementations but `const`. ([#2098], [#2107])

[#2092]: https://github.com/CosmWasm/cosmwasm/pull/2092
[#2098]: https://github.com/CosmWasm/cosmwasm/pull/2098
[#2107]: https://github.com/CosmWasm/cosmwasm/pull/2107

### Changed

- cosmwasm-std: Let `Timestamp::plus_nanos`/`::minus_nanos` use
  `Uint64::strict_add`/`::strict_sub` and document overflows. ([#2098], [#2107])

[#2098]: https://github.com/CosmWasm/cosmwasm/pull/2098
[#2107]: https://github.com/CosmWasm/cosmwasm/pull/2107

### Fixed

- cosmwasm-std: Correctly deallocate vectors that were turned into a `Region`
  via `release_buffer` ([#2062])

[#2062]: https://github.com/CosmWasm/cosmwasm/pull/2062

## [1.5.3]

### Changed

- cosmwasm-vm: Read `Region` from Wasm memory as bytes and convert to `Region`
  afterwards ([#2005])

[#2005]: https://github.com/CosmWasm/cosmwasm/pull/2005

## [1.5.2] - 2024-01-15

### Fixed

- cosmwasm-vm: Fix memory increase issue (1.3 -> 1.4 regression) by avoiding the
  use of a long running Wasmer Engine. ([#1978])

[#1978]: https://github.com/CosmWasm/cosmwasm/issues/1978

## [1.5.1] - 2024-01-10

### Fixed

- cosmwasm-vm: Fix CWA-2023-004.

### Added

- cosmwasm-vm: Add constructor `CacheOptions::new`

## [1.5.0] - 2023-10-31

### Added

- cosmwasm-std: Add `addr_make` and `with_prefix` for
  `cosmwasm_std::testing::MockApi` ([#1905]).
- cosmwasm-std: Add `abs` and `unsigned_abs` for `Int{64,128,256,512}`
  ([#1854]).
- cosmwasm-std: Add `From<Int{64,128,256}>` for `Int512`,
  `TryFrom<Int{128,256,512}>` for `Int64`, `TryFrom<Int{256,512}>` for `Int128`,
  `TryFrom<Int512>` for `Int256` and `Int256::from_i128` for const contexts
  ([#1861]).
- cosmwasm-std: Add `Int{64,128,256}::{checked_multiply_ratio, full_mul}`
  ([#1866])
- cosmwasm-std: Add `is_negative` for `Int{64,128,256,512}` ([#1867]).
- cosmwasm-std: Add `TryFrom<Uint{256,512}> for Uint64` and
  `TryFrom<Uint{A}> for Int{B}` where `A >= B` ([#1870]).
- cosmwasm-std: Add `to_json_{vec,binary,string}` and `from_json` and deprecate
  `to_{vec,binary}` in favor of `to_json_{vec,binary}` and `from_{slice,binary}`
  in favor of `from_json`. ([#1886])
- cosmwasm-std: Add `SignedDecimal` and `SignedDecimal256` ([#1807]).
- cosmwasm-vm: Allow float operations with NaN canonicalization ([#1864]).

[#1905]: https://github.com/CosmWasm/cosmwasm/pull/1905
[#1854]: https://github.com/CosmWasm/cosmwasm/pull/1854
[#1861]: https://github.com/CosmWasm/cosmwasm/pull/1861
[#1866]: https://github.com/CosmWasm/cosmwasm/pull/1866
[#1867]: https://github.com/CosmWasm/cosmwasm/pull/1867
[#1870]: https://github.com/CosmWasm/cosmwasm/pull/1870
[#1886]: https://github.com/CosmWasm/cosmwasm/pull/1886
[#1807]: https://github.com/CosmWasm/cosmwasm/pull/1807
[#1864]: https://github.com/CosmWasm/cosmwasm/pull/1864

### Changed

- cosmwasm-vm: Added `.module` extension to file names in the file system cache
  ([#1913]).

[#1913]: https://github.com/CosmWasm/cosmwasm/pull/1913

## [1.4.1] - 2023-10-09

## Fixed

- cosmwasm-vm: Fix a 1.3.x -> 1.4.0 regression bug leading to a _Wasmer runtime
  error: RuntimeError: out of bounds memory access_ in cases when the Wasm file
  is re-compiled and used right away. ([#1907])

[#1907]: https://github.com/CosmWasm/cosmwasm/pull/1907

### Changed

- cosmwasm-check: Use "=" for pinning the versions of cosmwasm-vm and
  cosmwasm-std dependencies. This ensures that you can use an older version of
  cosmwasm-check together with the VM of the same version by doing
  `cargo install cosmwasm-check@1.4.1`. A typical use case would be to check a
  contract with CosmWasm 1.4, 1.5 and 2.0. Note that other dependencies are
  still upgraded when using `cargo install` which may lead to API, behavioural
  or compiler incompatibilities. The
  [--locked](https://doc.rust-lang.org/cargo/commands/cargo-install.html#dealing-with-the-lockfile)
  feature allows you use the versions locked when the release was created.

## [1.4.0] - 2023-09-04

### Added

- cosmwasm-std: Implement `Not` for `Uint{64,128,256}` ([#1799]).
- cosmwasm-std: Add iterators for `Coins` ([#1806]).
- cosmwasm-std: Make `abs_diff` const for `Uint{256,512}` and
  `Int{64,128,256,512}`. It is now const for all integer types.
- cosmwasm-std: Implement `TryFrom<Decimal256>` for `Decimal` ([#1832])
- cosmwasm-std: Add `StdAck`. ([#1512])
- cosmwasm-std: Add new imports `db_next_{key, value}` for iterating storage
  keys / values only and make `Storage::{range_keys, range_values}` more
  efficient. This requires the `cosmwasm_1_4` feature to be enabled. ([#1834])
- cosmwasm-std: Add
  `DistributionQuery::{DelegationRewards, DelegationTotalRewards, DelegatorValidators}`.
  This requires the `cosmwasm_1_4` feature to be enabled. ([#1788])
- cosmwasm-std: Export module `cosmwasm_std::storage_keys` with
  `namespace_with_key`, `to_length_prefixed` and `to_length_prefixed_nested` to
  make it easier to use the strandard storage key layout documented in
  [STORAGE_KEYS.md](https://github.com/CosmWasm/cosmwasm/blob/v1.5.0/docs/STORAGE_KEYS.md)
  in other libraries such as cw-storage-plus or indexers. ([#1676])

[#1512]: https://github.com/CosmWasm/cosmwasm/issues/1512
[#1676]: https://github.com/CosmWasm/cosmwasm/pull/1676
[#1799]: https://github.com/CosmWasm/cosmwasm/pull/1799
[#1806]: https://github.com/CosmWasm/cosmwasm/pull/1806
[#1832]: https://github.com/CosmWasm/cosmwasm/pull/1832
[#1834]: https://github.com/CosmWasm/cosmwasm/pull/1834
[#1788]: https://github.com/CosmWasm/cosmwasm/pull/1788

### Changed

- cosmwasm-vm: Avoid using loupe for getting the `Module` size in the file
  system cache to prepare for the Wasmer 3 upgrade.
- cosmwasm-vm: When enabling `print_debug` the debug logs are now printed to
  STDERR instead of STDOUT by default ([#1667]).
- cosmwasm-vm: Add `Instance::set_debug_handler`/`unset_debug_handler` to allow
  customizing the handling of debug messages emitted by the contract ([#1667]).
- cosmwasm-vm: Upgrade Wasmer to version 4.1. ([#1674], [#1693], [#1701],
  [#1793])
- cosmwasm-check: Update clap dependency to version 4 ([#1677])
- cosmwasm-vm: Use `wasmparser` for initial validation instead of `parity-wasm`
  ([#1786])
- cosmwasm-std: Make constructors `Decimal{,256}::{percent,permille,bps}` const
- cosmwasm-std: Use new `db_next_key` import to make `skip` and `nth`
  implementation of `range` iterators more efficient. This requires the
  `cosmwasm_1_4` feature to be enabled. ([#1838])

[#1667]: https://github.com/CosmWasm/cosmwasm/pull/1667
[#1674]: https://github.com/CosmWasm/cosmwasm/pull/1674
[#1677]: https://github.com/CosmWasm/cosmwasm/pull/1677
[#1693]: https://github.com/CosmWasm/cosmwasm/pull/1693
[#1701]: https://github.com/CosmWasm/cosmwasm/pull/1701
[#1786]: https://github.com/CosmWasm/cosmwasm/pull/1786
[#1793]: https://github.com/CosmWasm/cosmwasm/pull/1793
[#1838]: https://github.com/CosmWasm/cosmwasm/pull/1838

## [1.3.3] - 2023-08-22

### Added

- cosmwasm-std: Implement `into_empty` for `QuerierWrapper`, `Deps` and
  `DepsMut`.

## [1.3.2] - 2023-08-15

### Fixed

- cosmwasm-std: Export `CoinFromStrError`, `CoinsError` and `DivisionError`

## [1.3.1] - 2023-07-26

### Fixed

- cosmwasm-std: Export `DelegatorWithdrawAddressResponse`,
  `DenomMetadataResponse` and `AllDenomMetadataResponse` which were added in
  `1.3.0` ([#1795]).

[#1795]: https://github.com/CosmWasm/cosmwasm/pull/1795

### Changed

- cosmwasm-std: Query responses are now exported, even if the corresponding
  cargo feature is not enabled ([#1795]).

## [1.3.0] - 2023-07-17

### Fixed

- cosmwasm-vm: Add missing cache stats increment when calling `pin`.

### Added

- cosmwasm-std: Implement `BankQuery::AllDenomMetadata` to allow querying all
  the denom metadata and `BankQuery::DenomMetadata` to query a specific one. In
  order to use this query in a contract, the `cosmwasm_1_3` feature needs to be
  enabled for the `cosmwasm_std` dependency. This makes the contract
  incompatible with chains running anything lower than CosmWasm `1.3.0`.
  ([#1647])
- cosmwasm-std: Add `DistributionQuery::DelegatorWithdrawAddress`. Also needs
  the `cosmwasm_1_3` feature (see above). ([#1593])
- cosmwasm-std: Add `DistributionMsg::FundCommunityPool`. Also needs the
  `cosmwasm_1_3` feature (see above). ([#1747])
- cosmwasm-std: Add `FromStr` impl for `Coin`. ([#1684])
- cosmwasm-std: Add `Coins` helper to handle multiple coins. ([#1687])
- cosmwasm-vm: Add `Cache::save_wasm_unchecked` to save Wasm blobs that have
  been checked before. This is useful for state-sync where we know the Wasm code
  was checked when it was first uploaded. ([#1635])
- cosmwasm-vm: Allow sign extension Wasm opcodes in static validation. This
  allows contracts to be compiled with Rust 1.70.0 and above. ([#1727])
- cosmwasm-std: Add trait functions `Storage::range_keys` and
  `Storage::range_values`. The default implementations just use
  `Storage::range`. Later this can be implemented more efficiently. ([#1748])
- cosmwasm-std: Add `Int64`, `Int128`, `Int256` and `Int512` signed integer
  types. ([#1718])

[#1593]: https://github.com/CosmWasm/cosmwasm/pull/1593
[#1635]: https://github.com/CosmWasm/cosmwasm/pull/1635
[#1647]: https://github.com/CosmWasm/cosmwasm/pull/1647
[#1684]: https://github.com/CosmWasm/cosmwasm/pull/1684
[#1687]: https://github.com/CosmWasm/cosmwasm/pull/1687
[#1718]: https://github.com/CosmWasm/cosmwasm/pull/1718
[#1727]: https://github.com/CosmWasm/cosmwasm/issues/1727
[#1747]: https://github.com/CosmWasm/cosmwasm/pull/1747
[#1748]: https://github.com/CosmWasm/cosmwasm/pull/1748

### Changed

- cosmwasm-vm: Add checks for table section of Wasm blob ([#1631]).
- cosmwasm-vm: Limit number of imports during static validation ([#1629]).
- cosmwasm-vm: Add target (triple + CPU features) into the module cache
  directory to avoid using modules compiled for a different system. Bump
  `MODULE_SERIALIZATION_VERSION` to "v6". ([#1664])
- cosmwasm-vm: Add `.wasm` extension to stored wasm files ([#1686]).

[#1629]: https://github.com/CosmWasm/cosmwasm/pull/1629
[#1631]: https://github.com/CosmWasm/cosmwasm/pull/1631
[#1664]: https://github.com/CosmWasm/cosmwasm/pull/1664
[#1686]: https://github.com/CosmWasm/cosmwasm/pull/1686

### Deprecated

- cosmwasm-storage: All exports are deprecated because this crate will be
  removed with CosmWasm 2.0 ([#1596]).

[#1596]: https://github.com/CosmWasm/cosmwasm/issues/1596

## [1.2.7] - 2023-06-19

### Added

- cosmwasm-std: Add `<<` and `<<=` implementation for `Uint{64,128,256,512}`
  types. ([#1723])
- cosmwasm-std: Add `Timestamp::{plus,minus}_{minutes, hours, days}`. ([#1729])
- cosmwasm-std: Add `Decimal::bps` and `Decimal256::bps` to create a decimal
  from a basis point value ([#1715]).

[#1723]: https://github.com/CosmWasm/cosmwasm/pull/1723
[#1729]: https://github.com/CosmWasm/cosmwasm/pull/1729
[#1715]: https://github.com/CosmWasm/cosmwasm/pull/1715

### Changed

- cosmwasm-std: Coin uses shorter `Coin { 123 "ucosm" }` format for Debug
  ([#1704])

[#1704]: https://github.com/CosmWasm/cosmwasm/pull/1704

## [1.2.6] - 2023-06-05

### Changed

- cosmwasm-vm: Bumped module serialization version from v4 to v5 to invalidate
  potentially corrupted caches caused by Rust update. See
  https://github.com/CosmWasm/wasmvm/issues/426 for more information. ([#1708])

[#1708]: https://github.com/CosmWasm/cosmwasm/pull/1708

## [1.2.5] - 2023-05-02

### Added

- cosmwasm-std: Implement `PartialEq` for `Addr == &Addr` and `&Addr == Addr` as
  well as `Event == &Event` and `&Event == Event` ([#1672]).
- cosmwasm-std: Add `#[must_use]` annotations to `Uint64`, `Uint128`, `Uint256`,
  `Uint512`, `Decimal` and `Decimal256` math operations ([#1678])

[#1672]: https://github.com/CosmWasm/cosmwasm/pull/1672
[#1678]: https://github.com/CosmWasm/cosmwasm/pull/1678

### Deprecated

- cosmwasm-std: The PartialEq implementations between `Addr` and `&str`/`String`
  are deprecated because they are not considered to be safe. In almost all cases
  you want to convert both sides of the equation to `Addr` first. If you really
  want to do a string comparison, use `Addr::as_str()` explicitly. ([#1671])

[#1671]: https://github.com/CosmWasm/cosmwasm/pull/1671

## [1.2.4] - 2023-04-17

### Fixed

- cosmwasm-vm: Add call depths limit

## [1.2.3] - 2023-03-22

- cosmwasm-vm: Use saturating increments for `Stats` fields to ensure we don't
  run into overflow issues.

## [1.2.2] - 2023-03-08

### Added

- cosmwasm-std: Add an IBC querier implementation to `testing::MockQuerier`
  ([#1620], [#1624]).
- cosmwasm-std: Add `#[must_use]` annotations to `Timestamp` math functions.

[#1620]: https://github.com/CosmWasm/cosmwasm/pull/1620
[#1624]: https://github.com/CosmWasm/cosmwasm/pull/1624

### Fixed

- all: Fix `backtraces` feature for newer versions of Rust. This still requires
  Rust nightly ([#1613]).
- cosmwasm-std: Add missing export `CheckedMultiplyFractionError` ([#1608]).

[#1608]: https://github.com/CosmWasm/cosmwasm/pull/1608
[#1613]: https://github.com/CosmWasm/cosmwasm/pull/1613

## [1.2.1] - 2023-01-30

### Added

- cosmwasm-std: Add `Decimal{,256}::to_uint_floor` and `::to_uint_ceil` for
  efficient and explicit decimal to uint conversion ([#1603]).

[#1603]: https://github.com/CosmWasm/cosmwasm/pull/1603

### Fixed

- cosmwasm-std: Make fields of `WeightedVoteOption` public to allow constructing
  it ([#1597]).

[#1597]: https://github.com/CosmWasm/cosmwasm/issues/1597

### Changed

- cosmwasm-std: Improve readability of `Debug` output for `Decimal` and
  `Decimal256` ([#1600]).

[#1600]: https://github.com/CosmWasm/cosmwasm/pull/1600

## [1.2.0] - 2023-01-24

### Added

- cosmwasm-std: Add `GovMsg::VoteWeighted`. In order to use this in a contract,
  the `cosmwasm_1_2` feature needs to be enabled for the `cosmwasm_std`
  dependency. This makes the contract incompatible with chains running versions
  of CosmWasm earlier than 1.2.0 ([#1481]).
- cosmwasm-std: Add `instantiate2_address` which allows calculating the
  predictable addresses for `MsgInstantiateContract2` ([#1437], [#1554]).
- cosmwasm-std: Add `WasmMsg::Instantiate2` (requires `cosmwasm_1_2`, see
  `GovMsg::VoteWeighted` above) to instantiate contracts at a predictable
  address ([#1436], [#1554])).
- cosmwasm-schema: In contracts, `cosmwasm schema` will now output a separate
  JSON Schema file for each entrypoint in the `raw` subdirectory ([#1478],
  [#1533]).
- cosmwasm-std: Upgrade `serde-json-wasm` dependency to 0.5.0 which adds map
  support to `to_vec`/`to_binary` and friends.
- cosmwasm-std: Implement `AsRef<[u8]>` for `Binary` and `HexBinary` ([#1550]).
- cosmwasm-std: Allow constructing `SupplyResponse` via a `Default`
  implementation ([#1552], [#1560]).
- cosmwasm-std: Add `Never` type which cannot be instantiated. This can be used
  as the error type for `ibc_packet_receive` or `ibc_packet_ack` to gain
  confidence that the implementations never errors and the transaction does not
  get reverted. ([#1513])
- cosmwasm-std: Add new `WasmQuery::CodeInfo` to get the checksum of a code ID
  ([#1561]).
- cosmwasm-vm: Add `Cache::remove_wasm` to remove obsolete Wasm blobs and their
  compiled modules.
- cosmwasm-std: Implement fraction multiplication and division. Assists with
  Uint & Decimal arithmetic and exposes methods for flooring/ceiling result
  ([#1485], [#1566]).

[#1436]: https://github.com/CosmWasm/cosmwasm/issues/1436
[#1437]: https://github.com/CosmWasm/cosmwasm/issues/1437
[#1478]: https://github.com/CosmWasm/cosmwasm/pull/1478
[#1481]: https://github.com/CosmWasm/cosmwasm/pull/1481
[#1485]: https://github.com/CosmWasm/cosmwasm/issues/1485
[#1513]: https://github.com/CosmWasm/cosmwasm/pull/1513
[#1533]: https://github.com/CosmWasm/cosmwasm/pull/1533
[#1550]: https://github.com/CosmWasm/cosmwasm/issues/1550
[#1552]: https://github.com/CosmWasm/cosmwasm/pull/1552
[#1554]: https://github.com/CosmWasm/cosmwasm/pull/1554
[#1560]: https://github.com/CosmWasm/cosmwasm/pull/1560
[#1561]: https://github.com/CosmWasm/cosmwasm/pull/1561
[#1566]: https://github.com/CosmWasm/cosmwasm/pull/1566

### Changed

- cosmwasm-vm: Avoid exposing OS specific file system errors in order to test
  cosmwasm-vm on Windows. This gives us confidence for integrating cosmwasm-vm
  in a libwasmvm build on Windows. This change is likely to be consensus
  breaking as error messages change. ([#1406])
- cosmwasm-vm: Use `Display` representation for embedding Wasmer
  `InstantiationError`s ([#1508]).

[#1406]: https://github.com/CosmWasm/cosmwasm/pull/1406
[#1508]: https://github.com/CosmWasm/cosmwasm/issues/1508

### Fixed

- cosmwasm-schema: Nested QueryMsg with generics is now supported by the
  QueryResponses macro ([#1516]).
- cosmwasm-schema: A nested QueryMsg no longer causes runtime errors if it
  contains doc comments.
- cosmwasm-std/cosmwasm-vm: Increase length limit for address conversion in
  `MockApi` to support addresses longer than 54 bytes ([#1529]).

[#1516]: https://github.com/CosmWasm/cosmwasm/issues/1516
[#1529]: https://github.com/CosmWasm/cosmwasm/issues/1529

## [1.1.9] - 2022-12-06

### Fixed

- cosmwasm-schema: Fix type fully qualified path to symbol `QueryResponses` in
  macro `cosmwasm_schema::generate_api!` ([#1527]).

[#1527]: https://github.com/CosmWasm/cosmwasm/issues/1527

## [1.1.8] - 2022-11-22

### Fixed

- cosmwasm-schema: Fix type params on `QueryMsg` causing a compiler error when
  used with the `QueryResponses` derive macro.

## [1.1.6] - 2022-11-16

### Added

- cosmwasm-std: Add `From` implementations to convert between
  `CanonicalAddr`/`Binary` as well as `CanonicalAddr`/`HexBinary` ([#1463]).
- cosmwasm-std: Add `From` implementations to convert `u8` arrays to
  `CanonicalAddr` ([#1463]).
- cosmwasm-std: Implement `PartialEq` between `CanonicalAddr` and
  `HexBinary`/`Binary` ([#1463]).

[#1463]: https://github.com/CosmWasm/cosmwasm/pull/1463

### Changed

- all: Bump a few dependency versions to make the codebase compile with
  `-Zminimal-versions` ([#1465]).
- cosmwasm-profiler: Package was removed 🪦. It served its job showing us that
  we cannot properly measure different runtimes for different Wasm opcodes.
- cosmwasm-schema: schema generation is now locked to produce strictly
  `draft-07` schemas
- cosmwasm-schema: `QueryResponses` derive now sets the `JsonSchema` trait bound
  on the generated `impl` block. This allows the contract dev to not add a
  `JsonSchema` trait bound on the type itself.

[#1465]: https://github.com/CosmWasm/cosmwasm/pull/1465

## [1.1.5] - 2022-10-17

### Added

- cosmwasm-std: Add `wrapping_add`, `wrapping_sub`, `wrapping_mul` and
  `wrapping_pow` to `Uint256`/`Uint512`.
- cosmwasm-schema: Better error messaging when attempting to compile schema
  generator for `wasm32`
- cosmwasm-vm: In the `secp256k1_verify`, `secp256k1_recover_pubkey`,
  `ed25519_verify` and `ed25519_batch_verify` import implementations we now exit
  early if the gas left is not sufficient to perform the operation.

### Changed

- cosmwasm-std: Remove `non_exhaustive` from IBC types `IbcChannelOpenMsg`,
  `IbcChannelConnectMsg` and `IbcChannelCloseMsg` in order to allow exhaustive
  matching over the possible scenarios without an unused fallback case
  ([#1449]).

[#1449]: https://github.com/CosmWasm/cosmwasm/pull/1449

## [1.1.4] - 2022-10-03

### Fixed

- cosmwasm-schema: Properly analyze schemas generated for `untagged` enums

## [1.1.3] - 2022-09-29

### Fixed

- cosmwasm-schema: `IntegrityError` is now public

## [1.1.2] - 2022-09-19

### Added

- cosmwasm-std: Add testing macro `assert_approx_eq!` for comparing two integers
  to be relatively close to each other ([#1417]).
- cosmwasm-std: Add `HexBinary` which is like `Binary` but encodes to hex
  strings in JSON. Add `StdError::InvalidHex` error case. ([#1425])

[#1417]: https://github.com/CosmWasm/cosmwasm/issues/1417
[#1425]: https://github.com/CosmWasm/cosmwasm/pull/1425

### Fixed

- cosmwasm-vm: Bump `MODULE_SERIALIZATION_VERSION` to "v4" because the module
  serialization format changed between Wasmer 2.2 and 2.3 ([#1426]).
- cosmwasm-schema: The `QueryResponses` derive macro now supports `QueryMsg`s
  with generics. ([#1429])

[#1426]: https://github.com/CosmWasm/cosmwasm/issues/1426
[#1429]: https://github.com/CosmWasm/cosmwasm/pull/1429

## [1.1.1] - 2022-09-15

### Fixed

- cosmwasm-schema: Using `QueryResponses` with a `QueryMsg` containing a
  unit-like variant will no longer crash. The different variant types in Rust
  are:
  ```rust
  enum QueryMsg {
      UnitLike,
      Tuple(),
      Struct {},
  }
  ```
  It's still recommended to only use struct variants, even if there are no
  fields.

### Changed

- cosmwasm-schema: It is no longer necessary to specify `serde` or `schemars` as
  a dependency in order to make `cosmwasm-schema` macros work.

## [1.1.0] - 2022-09-05

### Added

- cosmwasm-std: Implement PartialEq for `Binary` and `u8` arrays.
- cosmwasm-std: Add `Uint{64,128,256,512}::one`.
- cosmwasm-std: Add `Uint{64,128,256,512}::abs_diff` and
  `Decimal{,256}::abs_diff` ([#1334]).
- cosmwasm-std: Implement `From<Decimal> for Decimal256`.
- cosmwasm-std: Implement `Rem`/`RemAssign` for `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `checked_add`/`_sub`/`_div`/`_rem` for
  `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `pow`/`saturating_pow` for `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `ceil`/`floor` for `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `PartialEq` for reference on one side and owned value
  on the other for all `Uint` and `Decimal` types
- cosmwasm-std: Implement `saturating_add`/`sub`/`mul` for
  `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `BankQuery::Supply` to allow querying the total supply
  of a native token. In order to use this query in a contract, the
  `cosmwasm_1_1` feature needs to be enabled for the `cosmwasm_std` dependency.
  This makes the contract incompatible with chains running CosmWasm `1.0`.
  ([#1356])
- cosmwasm-std: Implement `MIN` const value for all `Uint` and `Decimal` types
- cosmwasm-std: Implement `checked_div_euclid` for `Uint256`/`Uint512`
- cosmwasm-std: Add `QuerierWrapper::query_wasm_contract_info` - this is just a
  convenience helper for querying `WasmQuery::ContractInfo`.
- cosmwasm-check: This is a new binary package that allows running various
  CosmWasm compatibility checks on compiled .wasm files. See
  https://crates.io/crates/cosmwasm-check for usage info.

[#1334]: https://github.com/CosmWasm/cosmwasm/pull/1334
[#1356]: https://github.com/CosmWasm/cosmwasm/pull/1356

### Changed

- cosmwasm-vm/cosmwasm-profiler: Upgrade Wasmer to 2.3.0.
- cosmwasm-std: Enable the `abort` feature by default. This provides more
  helpful panic messages via a custom panic handler.
- cosmwasm-std: Make `Decimal{,256}::DECIMAL_PLACES` a public `u32` value.
- cosmwasm-crypto: Bumped `k256` `0.10.4 -> 0.11` and `digest` `0.9 -> 0.10`
  ([#1374]).
- cosmwasm-vm: Rename features to capabilities, including
  1. `features_from_csv` to `capabilities_from_csv`;
  2. `CacheOptions::supported_features` to
     `CacheOptions::available_capabilities`;
  3. `MockInstanceOptions::supported_features` to
     `MockInstanceOptions::available_capabilities`
  4. `Instance::required_features` to `Instance::required_capabilities`
  5. `AnalysisReport::required_features` to
     `AnalysisReport::required_capabilities`.

[#1374]: https://github.com/CosmWasm/cosmwasm/pull/1374

### Deprecated

- cosmwasm-vm: The `check_contract` example was deprecated. Please use the new
  crate [cosmwasm-check](https://crates.io/crates/cosmwasm-check) instead
  ([#1371]).

[#1371]: https://github.com/CosmWasm/cosmwasm/issues/1371

## [1.0.0] - 2022-05-14

### Added

- cosmwasm-std: Export `DelegationResponse` ([#1301]).
- cosmwasm-std: When the new `abort` feature is enabled, cosmwasm-std installs a
  panic handler that aborts the contract and passes the panic message to the
  host. The `abort` feature can only be used when deploying to chains that
  implement the import. For this reason, it's not yet enabled by default.
  ([#1299])
- cosmwasm-vm: A new import `abort` is created to abort contract execution when
  requested by the contract. ([#1299])
- cosmwasm-std: Add new `ibc3` feature that allows to use IBC-Go V3 features,
  like version negotiation and exposing relayer address to the contract.
  Requires a compatible wasmd runtime (v0.27.0+) ([#1302])

[#1299]: https://github.com/CosmWasm/cosmwasm/pull/1299
[#1301]: https://github.com/CosmWasm/cosmwasm/pull/1301
[#1302]: https://github.com/CosmWasm/cosmwasm/pull/1302

---

The CHANGELOG for versions before 1.0.0 was moved to
[CHANGELOG-pre1.0.0.md](./CHANGELOG-pre1.0.0.md).

<!-- next-url -->

[unreleased]: https://github.com/CosmWasm/cosmwasm/compare/v3.0.1...HEAD
[3.0.1]: https://github.com/CosmWasm/cosmwasm/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/CosmWasm/cosmwasm/compare/v2.2.0...v3.0.0
[2.2.0]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.5...v2.2.0
[2.1.5]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.4...v2.1.5
[2.1.4]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.3...v2.1.4
[2.1.3]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.2...v2.1.3
[2.1.2]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.1...v2.1.2
[2.1.1]: https://github.com/CosmWasm/cosmwasm/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.8...v2.1.0
[2.0.8]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.7...v2.0.8
[2.0.7]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.6...v2.0.7
[2.0.6]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.5...v2.0.6
[2.0.5]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.4...v2.0.5
[2.0.4]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.3...v2.0.4
[2.0.3]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.2...v2.0.3
[2.0.2]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/CosmWasm/cosmwasm/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.9...v2.0.0
[1.5.9]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.8...v1.5.9
[1.5.8]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.7...v1.5.8
[1.5.7]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.6...v1.5.7
[1.5.6]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.5...v1.5.6
[1.5.5]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.4...v1.5.5
[1.5.4]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.3...v1.5.4
[1.5.3]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.2...v1.5.3
[1.5.2]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.1...v1.5.2
[1.5.1]: https://github.com/CosmWasm/cosmwasm/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.4.1...v1.5.0
[1.4.1]: https://github.com/CosmWasm/cosmwasm/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.3.3...v1.4.0
[1.3.3]: https://github.com/CosmWasm/cosmwasm/compare/v1.3.2...v1.3.3
[1.3.2]: https://github.com/CosmWasm/cosmwasm/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/CosmWasm/cosmwasm/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.7...v1.3.0
[1.2.7]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.6...v1.2.7
[1.2.6]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.5...v1.2.6
[1.2.5]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.4...v1.2.5
[1.2.4]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.3...v1.2.4
[1.2.3]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.2...v1.2.3
[1.2.2]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.1...v1.2.2
[1.2.1]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.9...v1.2.0
[1.1.9]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.8...v1.1.9
[1.1.8]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.6...v1.1.8
[1.1.6]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.5...v1.1.6
[1.1.5]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.4...v1.1.5
[1.1.4]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.3...v1.1.4
[1.1.3]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/CosmWasm/cosmwasm/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-rc.0...v1.0.0
