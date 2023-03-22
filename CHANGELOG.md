# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
  we cannot properly measure different runtimes for differet Wasm opcodes.
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

## [1.0.0-rc.0] - 2022-05-05

### Fixed

- cosmwasm-std: Upgrade `serde-json-wasm` to 0.4.0 to fix u128/i128
  serialization of `to_vec`/`to_binary` in some cases ([#1297]).

[#1297]: https://github.com/CosmWasm/cosmwasm/pull/1297

### Added

- cosmwasm-std: Implement `checked_multiply_ratio` for
  `Uint64`/`Uint128`/`Uint256`
- cosmwasm-std: Implement `checked_from_ratio` for `Decimal`/`Decimal256`
- cosmwasm-std: Implement `Div`/`DivAssign` for `Decimal`/`Decimal256`.
- cosmwasm-vm: Add feature `allow_interface_version_7` to run CosmWasm 0.16
  contracts in modern hosts. Be careful if you consider using this!

### Changed

- all: Updated Rust edition to 2021
- cosmwasm-std: Rename `SubMsgExecutionResponse` to `SubMsgResponse`.
- cosmwasm-crypto: Update dependency `k256` to ^0.10.4.
- cosmwasm-vm: `BackendError` was changed to `non_exhaustive` for future
  extension; `BackendError` now implements `PartialEq` for easier test code; the
  `msg` in `BackendError::Unknown` became non-optional because it was always
  set; the argument in `BackendError::unknown`/`::user_err` was change to
  `impl Into<String>` to avoid unnecessary clones.

### Deprecated

- cosmwasm-std: `SubMsgExecutionResponse` is deprecated in favor of the new
  `SubMsgResponse`.

### Removed

- cosmwasm-std: Remove `Pair` which was previously deprecated. Use `Record`
  instead. ([#1282])

[#1282]: https://github.com/CosmWasm/cosmwasm/issues/1282

## [1.0.0-beta8] - 2022-04-06

### Added

- cosmwasm-std: Implement `MulAssign` for `Decimal`/`Decimal256`.
- cosmwasm-std: Implement `is_zero`/`atomics`/`decimal_places` as const for Uint
  and Decimal types.
- cosmwasm-std: Implement `new` and `raw` const constructors for
  `Decimal`/`Decimal256`.

### Changed

- all: Drop support for Rust versions lower than 1.56.1.
- cosmwasm-std: `MockQuerier` now supports adding custom behaviour for handling
  Wasm queries via `MockQuerier::update_wasm` ([#1050]).

[#1050]: https://github.com/CosmWasm/cosmwasm/pull/1050

### Fixed

- cosmwasm-std: `Api::addr_validate` now requires inputs to be normalized.
- cosmwasm-vm: The `addr_validate` import now requires inputs to be normalized.

## [1.0.0-beta7] - 2022-03-22

### Added

- cosmwasm-std: Implement `Decimal{,256}::checked_mul` and
  `Decimal{,256}::checked_pow`.
- cosmwasm-std: Implement `Sub`/`SubAssign` for `Uint64`.
- cosmwasm-std: Implement `Mul`/`MulAssign` for `Uint64`.
- cosmwasm-std: Implement `RemAssign` for
  `Uint64`/`Uint128`/`Uint256`/`Uint512`.
- cosmwasm-std: Implement `pow`/`checked_pow` for `Uint64`/`Uint128`/`Uint512`.
- cosmwasm-std: Implement `SubAssign`/`AddAssign` for `Decimal`/`Decimal256`.
- cosmwasm-crypto: Upgrade ed25519-zebra to version 3.

### Changed

- cosmwasm-vm: Upgrade Wasmer to 2.2.1.

## [1.0.0-beta6] - 2022-03-07

### Added

- cosmwasm-std: Implement `ops::Rem` for `Uint{64,128,256,512}`.

### Changed

- cosmwasm-std: Change type of `Reply::result` from `ContractResult` to the new
  `SubMsgResult`. Both types are equal when serialized but `ContractResult` is
  documented to be the result of a contract execution, which is not the case
  here. ([#1232])
- cosmwasm-vm: Upgrade Wasmer to 2.2.0 and bump `MODULE_SERIALIZATION_VERSION`
  to "v3-wasmer1". ([#1224])

[#1224]: https://github.com/CosmWasm/cosmwasm/pull/1224
[#1232]: https://github.com/CosmWasm/cosmwasm/pull/1232

## [1.0.0-beta5] - 2022-02-08

### Changed

- all: Drop support for Rust versions lower than 1.54.0.
- cosmwasm-std: The `Debug` implementation of `Binary` now produces a hex string
  instead of a list of bytes ([#1199]).
- cosmwasm-std: Pin uint version to 0.9.1 in order to maintain a reasonably low
  MSRV.
- cosmwasm-std: Add missing `Isqrt` export ([#1214]).

[#1199]: https://github.com/CosmWasm/cosmwasm/issues/1199
[#1214]: https://github.com/CosmWasm/cosmwasm/issues/1214

### Fixed

- cosmwasm-vm: Fix `AddAssign` implementation of `GasInfo`.
- cosmwasm-vm: Bump `MODULE_SERIALIZATION_VERSION` to "v2" because the module
  serialization format changed between Wasmer 2.0.0 and 2.1.x.

## [1.0.0-beta4] - 2021-12-23

### Changed

- cosmwasm-vm: `wasmer` version bumped `2.1.0 -> 2.1.1`

### Fixed

- cosmwasm-vm: Remove system-dependent stacktrace from `VmError::RuntimeErr`
  (fixes CWA-2021-003).

## [1.0.0-beta3]

### Added

- cosmwasm-std: New const methods `Uint64::to_be_bytes`/`::to_le_bytes`.
- cosmwasm-vm: The check_contracts tool now has a `--supported-features` option
  that defaults to "iterator,staking,stargate".
- cosmwasm-vm: The default `singlepass` compiler is now supported on 64-bit
  Windows.
- cosmwasm-std: Add missing `DivideByZeroError` export.
- cosmwasm-std: Implement `std::iter::Sum` for `Decimal` and `Decimal256`.

### Changed

- all: Drop support for Rust versions lower than 1.53.0.
- cosmwasm-std: The balance argument from `mock_dependencies` was removed.
  Remove `&[]` if you don't need a contract balance or use the new
  `mock_dependencies_with_balance` if you need a balance.
- cosmwasm-vm: Unlock cache mutex before module instantiation.
- cosmwasm-vm: `wasmer` version bumped `2.0.0 -> 2.1.0`

### Removed

- cosmwasm-std: Remove the macros `create_entry_points` and
  `create_entry_points_with_migration` in favour of the new, more flexible entry
  point system introduced in CosmWasm 0.14.

## [1.0.0-beta] - 2021-10-11

### Added

- cosmwasm-std: Add new `WasmQuery::ContractInfo` variant to get metadata about
  the contract, like `code_id` and `admin`.
- cosmwasm-std: New field `Env::transaction` containing info of the transaction
  the contract call was executed in.
- cosmwasm-std: Implement `ops::Mul` for `Decimal` and `Decimal256`.
- cosmwasm-std: New const methods `Uint128::to_be_bytes`/`::to_le_bytes`.
- cosmwasm-std: New const conversion methods `Uint256::from_uint128` and
  `Uint512::from_uint256`.
- cosmwasm-std: New getters `Decimal{,256}::atomics()` and
  `Decimal{,256}::decimal_places()`.
- cosmwasm-std: New constructors `Decimal{,256}::from_atomics`.
- cosmwasm-std: New `Uint128::checked_pow`.
- cosmwasm-std: New macros `ensure!`, `ensure_eq!` and `ensure_ne!` allow
  requirement checking that return errors instead of panicking ([#1103]).

[#1103]: https://github.com/CosmWasm/cosmwasm/issues/1103

### Changed

- cosmwasm-std: Make `iterator` a required feature if the `iterator` feature
  flag is set (enabled by default).
- cosmwasm-vm: Increase `MAX_LENGTH_HUMAN_ADDRESS` from 90 to 256 in order to
  support longer address formats than bech32.
- cosmwasm-std: Make `CustomQuery` a subtrait of `Clone`, i.e. types that
  implement `CustomQuery` need to be `Clone`able.
- cosmwasm-std: Add generic for custom query type to `QuerierWrapper`, `Deps`,
  `DepsMut` and `OwnedDeps`. Merge `QuerierWrapper::custom_query` into the now
  fully typed `QuerierWrapper::query`.
- cosmwasm-std: Add generic type `Q` for the custom query request type to
  `do_instantiate`, `do_execute`, `do_migrate`, `do_sudo`, `do_reply`,
  `do_query`, `ibc_channel_open`, `ibc_channel_connect`, `ibc_channel_close`,
  `ibc_packet_receive`, `ibc_packet_ack` and `ibc_packet_timeout`.
- cosmwasm-std: In `Decimal` change `Fraction<u128>` to `Fraction<Uint128>`,
  such that `Decimal::numerator` and `::denominator` now return `Uint128`.
- cosmwasm-std: Make methods `Uint256::to_be_bytes`/`::to_le_bytes` const.
- cosmwasm-std: Make methods `Uint512::to_be_bytes`/`::to_le_bytes` const.
- cosmwasm-std: Make method `Uint512::from_le_bytes` const.
- cosmwasm-std: Rename `Pair` to `Record`. `Pair` is now an alias for `Record`
  and deprecated. ([#1108])
- cosmwasm-vm: Bump required marker export `interface_version_7` to
  `interface_version_8`.
- cosmwasm-vm: Increase cost per Wasm operation from 1 to 150_000 and adjust
  crypto API gas cost based on the target of 1 Teragas per millisecond.
- cosmwasm-std: Deprecate the macros `create_entry_points` and
  `create_entry_points_with_migration` in favour of the new, more flexible entry
  point system introduced in CosmWasm 0.14.

### Removed

- cosmwasm-std: Remove `HumanAddr` (deprecated since 0.14). Use `String`
  instead.
- cosmwasm-std: Remove `KV` (deprecated since 0.14). Use `Pair` instead.

[#1108]: https://github.com/CosmWasm/cosmwasm/issues/1108

## [0.16.2] - 2021-09-07

### Added

- cosmwasm-std: Implement `Mul` and `MulAssign` for `Uint128`.
- cosmwasm-std: Implement `FromStr` for `Uint128`, `Uint256`, and `Uint512`.
- cosmwasm-std: Make `Uint256::from_le_bytes`, `::from_be_bytes` and `::new`
  const.
- cosmwasm-std: Added the `Decimal256` type with 18 decimal places.

### Changed

- cosmwasm-std: Implement `Decimal::from_ratio` using full uint128
  multiplication to support a wider range of input values.
- cosmwasm-std: `Decimal::from_ratio` now accepts any types that implement
  `Into<Uint128>` rather than `Into<u128>`.
- cosmwasm-crypto: Update dependency `k256` to ^0.9.6.
- cosmwasm-std: Add enum cases `Shl` to `OverflowOperation` (breaking; [#1071]).

[#1071]: https://github.com/CosmWasm/cosmwasm/pull/1071

### Fixed

- cosmwasm-std: Fixed a bug where `Uint*` types wouldn't handle formatting
  options when formatted with `std::fmt::Display`.

## [0.16.1] - 2021-08-31

### Added

- cosmwasm-std: Added `From<Addr>` and `From<&Addr>` conversions for
  `Cow<Addr>`.
- cosmwasm-std: Added new `Uint256` and `Uint512` types.
- cosmwasm-std: Added implementations of `Isqrt` (integer square root) for
  `Uint64`, `Uint128`, `Uint256`, and `Uint512`.
- cosmwasm-std: Exposed `Uint{64, 128, 256}::full_mul` for full multiplication
  that cannot overflow.

### Changed

- cosmwasm-std: In `ExternalApi::addr_validate` and `::addr_canonicalize` do not
  send too long inputs to VM to avoid terminating contract execution. Errors are
  returned instead now.
- cosmwasm-std: Add enum cases `Shr` to `OverflowOperation` (breaking; [#1059]).

[#1059]: https://github.com/CosmWasm/cosmwasm/pull/1059

## [0.16.0] - 2021-08-05

### Added

- cosmwasm-std: Added the `IbcChannelOpenMsg`, `IbcChannelConnectMsg`,
  `IbcChannelCloseMsg`, `IbcPacketReceiveMsg`, `IbcPacketAckMsg`, and
  `IbcPacketTimeoutMsg` types for use with corresponding IBC entrypoints.
- cosmwasm-std::testing: New mocking helpers for IBC channel msg types:
  `mock_ibc_channel_open_init`, `mock_ibc_channel_open_try`,
  `mock_ibc_channel_connect_ack`, `mock_ibc_channel_connect_confirm`,
  `mock_ibc_channel_close_init`, `mock_ibc_channel_close_confirm`.
- cosmwasm-std::testing: Added `mock_ibc_packet_timeout` since
  `mock_ibc_packet_ack` is no longer usable for creating mock data for
  `ibc_packet_timeout`.
- cosmwasm-std: New `Attribute::new` constructor that does the same thing as
  `attr`.
- cosmwasm-std::testing: Added `mock_wasm_attr` when you really need to create
  an `Attribute` with a key starting with `_` in test code.
- cosmwasm-std: Renamed `IBCAcknowledgementWithPacket` -> `IbcPacketAckMsg` to
  remove an unneeded level of indirection.
- cosmwasm-std: Added `Event::add_attributes` for bulk adding attributes to an
  `Event` struct.
- cosmwasm-std: Added `Addr::into_string` for explicit conversion

### Changed

- cosmwasm-vm: The `Checksum::to_hex` function signature was changed from
  `to_hex(&self) -> String` to `to_hex(self) -> String`.
- cosmwasm-std: The `attr` function now accepts types that implement
  `Into<String>` rather than `ToString`.
- cosmwasm-std, cosmwasm-vm, cosmwasm-storage: The `iterator` feature is now
  enabled by default.
- cosmwasm-std: Make `MockApi::canonical_length` private.
- cosmwasm-vm: Make `MockApi::canonical_length` private.
- cosmwasm-vm: Bump required marker export `interface_version_6` to
  `interface_version_7`.
- cosmwasm-std, cosmwasm-vm: Entrypoints `ibc_channel_open`,
  `ibc_channel_connect`, `ibc_channel_close`, `ibc_packet_receive`,
  `ibc_packet_ack`, `ibc_packet_timeout` now each accept a corresponding `Msg`
  value that wraps around channels, packets and acknowledgements.
- cosmwasm-std/cosmwasm-vm: Increase canonical address lengths up to 64 bytes.
- cosmwasm-std/cosmwasm-vm: In `MockApi`, increase max length of supported human
  addresses from 24 bytes to 54 bytes by using a longer canonical
  representation. This allows you to insert typical bech32 addresses in tests.
  ([#995])
- cosmwasm-std::testing: `mock_ibc_packet_recv` function now returns an
  `IbcPacketReceiveMsg`, `mock_ibc_packet_ack` requires an acknowledgement to be
  passed and returns an `IbcPacketAckMsg`.
- cosmwasm-std: `IbcBasicResponse` and `IbcReceiveResponse` now both support
  custom events via the `events` field.
- cosmwasm-std: `attr` (and `Attribute::new`) will now panic in debug builds if
  the attribute's key starts with an underscore. These names are reserved and
  could cause problems further down the line.
- cosmwasm-std: `Response`, `IbcBasicResponse` and `IbcReceiveResponse` can no
  longer be constructed using struct literals. Use constructors like
  `Response::new` to construct empty structs and appropriate builder-style
  methods to set fields (`response.add_message`, `response.set_data`, etc).
- cosmwasm-std: `Event`, `IbcChannel`, `IbcPacket`, `IbcAcknowledgement` have
  been marked `non_exhaustive` (can't be constructed using a struct literal by
  downstream code).
- cosmwasm-std: `Event::attr` has been renamed to `Event::add_attribute` for
  consistency with other types like `Response`.
- cosmwasm-vm: `Instance::required_features` changed from a property to a getter
  method.
- cosmwasm-vm: Add `required_features` field to `AnalysisReport` which is
  returned by `Cache::analyze`.
- cosmwasm-vm: The VM now checks that exactly one `interface_version_*` marker
  export is set. For `interface_version_5` and `interface_version_6` (CosmWasm
  0.14–0.15) more specific error messages were added.

[#995]: https://github.com/CosmWasm/cosmwasm/pull/995

### Removed

- cosmwasm-std::testing: `mock_ibc_channel` is now private. Use
  `mock_ibc_channel_open`, `mock_ibc_channel_connect`, or
  `mock_ibc_channel_close` instead.

## [0.15.2] - 2021-07-21

### Fixed

- cosmwasm-std: Export `VoteOption` as a top-level type.

## [0.15.1] - 2021-07-20

### Fixed

- cosmwasm-std: Export `GovMsg` as a top-level type of the crate.

## [0.15.0] - 2021-06-24

### Added

- cosmwasm-std: Implement `Sub` and `SubAssign` for `Uint128`
- cosmwasm-std: Implement custom events for contract execution results
- cosmwasm-std: Add `CosmosMsg::Gov` for voting on governance proposals.
- cosmwasm-storage: Implement `Storage` for `PrefixedStorage` and
  `ReadonlyPrefixedStorage`. NOTE: Calling `set` or `remove` on
  `ReadonlyPrefixedStorage` will panic!

### Removed

- cosmwasm-std: Make `Uint128` inner field private ([#905])
- cosmwasm-std: Remove `Context` - deprecated in previous release
- cosmwasm-std: Remove `HandleResponse`, `InitResponse`, and `MigrateResponse` -
  deprecated in previous release
- cosmwasm-crypto: Remove `ed25519::MESSAGE_MAX_LEN`, `ed25519::BATCH_MAX_LEN`
  and message length verification as this should not be a concern of
  `cosmwasm-crypto`.

[#905]: https://github.com/CosmWasm/cosmwasm/issues/905

### Changed

- cosmwasm-std: Rename the `send` function parameter to `funds` in `WasmMsg` for
  consistency with the wasmd message types.
- cosmwasm-vm: Increase read limit of contract execution results from 100,000
  bytes to 64 MiB. JSON deserializers should have their own limit to protect
  against large deserializations.
- cosmwasm-vm: Create `VmError::DeserializationLimitExceeded`; Add limit
  argument to `from_slice`; Increase deserialization limit of contract execution
  results from 100,000 bytes to 256 KiB. This probably only affects internal
  testing as well as integration tests of smart contracts.
- cosmwasm-vm: More accurate error messages for op codes related to bulk memory
  operations, reference types, SIMD and the Threads extension.
- cosmwasm-vm: Update `wasmer` to `2.0.0`
- cosmwasm-vm: ED25519 message length and batch length limits are now hardcoded
  in `cosmwasm-vm` itself instead of being imported from `cosmwasm-crypto`.
- cosmwasm-vm: Filesystem storage layout now distinguishes clearly between state
  and cache.
- cosmwasm-std: Add enum case `ReplyOn::Never`; Remove default implementation of
  `ReplyOn` as there is no natural default case anymore ([#961]).
- cosmwasm-std: Merge `messages` and `submessages` into one list, using
  `ReplyOn::Never` to model the "fire and forget" semantics ([#961]).
- cosmwasm-std: Add `SubMsg` constructors: `::new()`, `::reply_on_error()`,
  `::reply_on_success()`, `::reply_always()`; Add `with_gas_limit` to add a gas
  limit to any those constructors ([#961]).
- cosmwasm-std: Change `Event`'s constructor - it no longer takes a vector of
  attributes and instead constructs an empty one
- cosmwasm-std: Rename `Event.kind` to `Event.ty`.
- cosmwasm-std: Rename `SubcallResponse` to `SubMsgExecutionResponse`.
- contracts: Rename `ReflectSubCall` to `ReflectSubMsg` and `SubCallResult` to
  `SubCallMsg` in the `reflect` contract.
- cosmwasm-std: Rename the `subcall` module to `submessages`.
- cosmwasm-vm: Bump required marker export `cosmwasm_vm_version_5` to
  `interface_version_6`.
- cosmwasm-std: `IbcAcknowledgement` is renamed to
  `IbcAcknowledgementWithPacket` as it contains both data elements. ([#975])
- cosmwasm-std: `IbcAcknowledgementWithPacket.acknowledgement` is no longer
  simply `Binary`, but a new `IbcAcknowledgement` structure, which contains one
  field - `data: Binary`. This change was made to allow us to handle future
  changes to IBC in a non-contract-breaking way. ([#975])

[#961]: https://github.com/CosmWasm/cosmwasm/pull/961
[#975]: https://github.com/CosmWasm/cosmwasm/pull/975

### Fixed

- comswasm-vm: Whitelisted the `i64.extend32_s` operation.

## [0.14.1] - 2021-06-14

### Added

- cosmwasm-std: Add `Timestamp::minus_seconds` and `::minus_nanos`.
- cosmwasm-std: Add `Addr::as_bytes`
- cosmwasm-std: Implement `std::ops::Sub` for `math::Decimal`
- cosmwasm-std: Add `Timestamp::seconds` and `Timestamp::subsec_nanos`.
- cosmwasm-std: Implement division for `Decimal / Uint128`
- cosmwasm-std: Add `math::Decimal::sqrt`

### Fixed

- cosmwasm-std: Fix `Uint64::multiply_ratio` and `Uint128::multiply_ratio` so
  that internal multiplication cannot cause an unnecessary overflow. ([#920])

[#920]: https://github.com/CosmWasm/cosmwasm/issues/920

## [0.14.0] - 2021-05-03

### Added

- cosmwasm-crypto: Add `ed25519_batch_verify`, EdDSA ed25519 batch signature
  verification scheme for Tendermint signatures and public keys formats.
  ([#788])
- cosmwasm-crypto: Add `ed25519_verify`, EdDSA ed25519 signature verification
  scheme for Tendermint signature and public key formats. ([#771])
- cosmwasm-crypto: New crypto-related crate. Add `secp256k1_verify`, ECDSA
  secp256k1 signature verification scheme for Cosmos signature and public key
  formats. ([#780])
- cosmwasm-vm: Add PinnedMemoryCache. ([#696])
- cosmwasm-vm: The new `Cache::analyze` provides a static analyzis of the Wasm
  bytecode. This is used to tell the caller if the contract exposes IBC entry
  points. ([#736])
- cosmwasm-vm: Added new `stargate` feature flag to enable new stargate and ibc
  features ([#692], [#716])
- cosmwasm-vm: (requires `stargate`) call into 6 new ibc entry points if exposed
  by contract ([#692], [#716])
- cosmwasm-std: Added new `stargate` feature flag to enable new stargate and ibc
  features ([#692], [#706])
- cosmwasm-std: (requires `stargate`) Added new `CosmosMsg::Stargate` message
  type to dispatch protobuf-encoded message (contract must know proto schema)
  ([#706])
- cosmwasm-std: (requires `stargate`) Added new `QueryRequest::Stargate` message
  type to dispatch protobuf-encoded queries (contract must know proto schema for
  request and response) ([#706])
- cosmwasm-std: (requires `stargate`) Added new `CosmosMsg::Ibc(IbcMsg)` message
  type to use ibctransfer app or send raw ics packets (if contract has ibc entry
  points) ([#692], [#710])
- cosmwasm-std: Add mutable helper methods to `InitResponse`, `MigrateResponse`
  and `HandleResponse` which make `Context` obsolete.
- contracts: added new `ibc-reflect` contract that receives channels and assigns
  each an account to redispatch. Similar idea to ICS27/Interchain Accounts (but
  different implementation) ([#692], [#711], [#714])
- cosmwasm-std: Added new `WasmMsg::Migrate` variant that allows one contract
  (eg. multisig) be the admin and migrate another contract ([#768])
- cosmwasm-std: Added optional `system` entry point that can only be called by
  native (blockchain) modules to expose admin functionality if desired. ([#793])
- cosmwasm-std: Add extra field `submessages` to `Response`, such that you can
  get a callback from these messages after their execution (success or failure).
  ([#796])
- cosmwasm-std: Added `reply` entry point that will receive all callbacks from
  submessages dispatched by this contract. This is only required if contract
  returns "submessages" (above). ([#796])
- cosmwasm-std: Implement `From<Uint128> for String`, `From<Uint128> for u128`
  as well as `From<u{32,16,8}> for Uint128`.
- cosmwasm-std: Create new address type `Addr`. This is human readable (like
  `HumanAddr`) but is immutable and always contains a valid address ([#802]).
- cosmwasm-vm: Add import `addr_validate` ([#802]).
- cosmwasm-std: Add `BankMsg::Burn` variant when you want the tokens to
  disappear ([#860])
- cosmwasm-std: Create `Fraction<T>` trait to represent a fraction `p`/`q` with
  integers `p` and `q`. `Decimal` now implements `Fraction<u128>`, which
  provides public getters `::numerator()` and `::denominator()`.
- cosmwasm-std: Add `Decimal::inv` that returns `1/d` for decimal `d`.
- cosmwasm-vm: Add `Cache::metrics` to expose internal data for monitoring
  purposes ([#763]).
- cosmwasm-std: Implement `PartialOrd` and `Ord` for `Binary` using the same
  lexicographical ordering as implemented by `Vec<u8>`.
- cosmwasm-std: Implement `PartialOrd` and `Ord` for `Addr` using the same
  lexicographical ordering as implemented by `String`.
- cosmwasm-std: Added new `WasmMsg::UpdateAdmin` variant that allows an admin
  contract (eg. multisig) to set another admin ([#900])
- cosmwasm-std: Added new `WasmMsg::ClearAdmin` variant that allows an admin
  contract (eg. multisig) to clear the admin, to prevent future migrations
  ([#900])
- cosmwasm-std: Implement `Display for Coin` ([#901]).
- cosmwasm-std: Create `Uint64` analogously to `Uint128` with string
  serialization allowing the use of the full uint64 range in JSON clients that
  use float numbers, such as JavaScript and jq.
- cosmwasm-std: Create const functions `Uint64::new` and `Uint128::new` to
  create instances in a const context.

[#692]: https://github.com/CosmWasm/cosmwasm/issues/692
[#706]: https://github.com/CosmWasm/cosmwasm/pull/706
[#710]: https://github.com/CosmWasm/cosmwasm/pull/710
[#711]: https://github.com/CosmWasm/cosmwasm/pull/711
[#714]: https://github.com/CosmWasm/cosmwasm/pull/714
[#716]: https://github.com/CosmWasm/cosmwasm/pull/716
[#763]: https://github.com/CosmWasm/cosmwasm/issues/763
[#768]: https://github.com/CosmWasm/cosmwasm/pull/768
[#793]: https://github.com/CosmWasm/cosmwasm/pull/793
[#796]: https://github.com/CosmWasm/cosmwasm/pull/796
[#802]: https://github.com/CosmWasm/cosmwasm/pull/802
[#860]: https://github.com/CosmWasm/cosmwasm/pull/860
[#900]: https://github.com/CosmWasm/cosmwasm/pull/900
[#901]: https://github.com/CosmWasm/cosmwasm/pull/901

### Changed

- contracts: Rename `HandleMsg` to `ExecuteMsg`.
- all: Rename `handle` entry point to `execute`.
- all: Rename `init` entry point to `instantiate`.
- all: Rename `system` entry point to `sudo`.
- all: Drop support for Rust versions lower than 1.51.0.
- all: The `query` and `execute` entry points are now optional. It is still
  highly recommended to implement and expose them in almost any use case though.
- all: Change the encoding of the key/value region of the `db_next` import to a
  more generic encoding that supports an arbitrary number of sections. This
  encoding can then be reused for other multi value regions.
- all: Remove the `info: MessageInfo` argument from the `migrate` entry point
  ([#690]).
- cosmwasm-std: Remove `from_address` from `BankMsg::Send`, as it always sends
  from the contract address, and this is consistent with other `CosmosMsg`
  variants.
- cosmwasm-std: Remove the previously deprecated `InitResult`, `HandleResult`,
  `MigrateResult` and `QueryResult` in order to make error type explicit and
  encourage migration to custom errors.
- cosmwasm-std: Add a `data` field to `InitResponse` the same way as in
  `MigrateResponse` and `HandleResponse`.
- cosmwasm-std: Rename `MessageInfo::sent_funds` to `MessageInfo::funds`.
- cosmwasm-std: Merge response types `InitResponse`, `HandleResponse` and
  `MigrateResponse` into the new `Response`.
- cosmwasm-std: Remove `Default` implementation from `HumanAddr`,
  `CanonicalAddr`, `ContractInfo`, `MessageInfo`, `BlockInfo` and `Env`. If you
  need one of those, you're probably doing something wrong.
- cosmwasm-std: Make `label` in `WasmMsg::Instantiate` non-optional to better
  match the Go/database format.
- cosmwasm-std: Add new field `admin` to `WasmMsg::Instantiate` to fully support
  `MsgInstantiateContract` from `x/wasm` ([#861]).
- cosmwasm-std: `Binary::to_array` is now generic over the array length instead
  of the output type. As a consequence the obsolete type `ByteArray` was
  removed. The array length is not restricted to 0-64 anymore.
- cosmwasm-std: Use const generics to implement `From<&[u8; LENGTH]> for Binary`
  and `From<[u8; LENGTH]> for Binary`, such that the array length is not
  restricted to 0-64 anymore.
- cosmwasm-vm: Avoid serialization of Modules in `InMemoryCache`, for
  performance. Also, remove `memory_limit` from `InstanceOptions`, and define it
  instead at `Cache` level (same memory limit for all cached instances).
  ([#697])
- cosmwasm-std: Rename type `KV` to `Pair` in order to comply to naming
  convention as enforced by clippy rule `upper_case_acronyms` from Rust 1.51.0
  on.
- cosmwasm-std: `ContractInfo::address` and `MessageInfo::sender` are now of
  type `Addr`. The value of those fields is created by the host and thus valid.
- cosmwasm-vm: Bump required marker export `cosmwasm_vm_version_4` to
  `interface_version_5`.
- cosmwasm-vm: Rename trait `Api` to `BackendApi` to better express this is the
  API provided by the VM's backend (i.e. the blockchain).
- cosmwasm-vm: Rename imports to `addr_canonicalize` and `addr_humanize`
  ([#802]).
- cosmwasm-vm: Replace types `HumanAddr`/`CanonicalAddr` with
  `&str`/`String`/`&[u8]`/`Vec<u8>` in the methods of `BackendApi`. The address
  types belong in the contract development and the backend operates on raw
  strings and binary anyways.
- contracts: `reflect` contract requires `stargate` feature and supports
  redispatching `Stargate` and `IbcMsg::Transfer` messages ([#692])
- cosmwasm-std: The arithmetic methods of `Uint128` got a huge overhaul, making
  them more consistent with the behaviour of the Rust primitive types. Thank you
  [@yihuang] for bringing this up and for the great implementation. ([#853])
  1.  `Uint128` got the new functions `checked_add`, `checked_sub`,
      `checked_mul`, `checked_div`, `checked_div_euclid`, `checked_rem`,
      `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_pow`,
      `saturating_add`, `saturating_sub`, `saturating_mul` and `saturating_pow`
      which match their equivalent in [u128] except that instead of `Option` the
      checked methods return a `Result` with an `OverflowError` or
      `DivideByZeroError` that carries a few debug information and can directly
      be converted to `StdError`/`StdResult` by using the `?` operator.
  2.  `StdError::Underflow` and `StdError::underflow` were removed in favour of
      `StdError::Overflow`. `StdError::DivideByZeroError` was added.
  3.  The `-` operator (`impl ops::Sub<Uint128> for Uint128`) was removed
      because it returned a `StdResult` instead of panicking in the case of an
      overflow. This behaviour was inconsistent with `+` and the Rust standard
      library. Please use the explicit `*_sub` methods introduced above. In a
      couple of releases from now, we want to introduce the operator again with
      panicking overflow behaviour ([#858]).
- cosmwasm-std: Replace `HumanAddr` with `String` in `BankQuery`, `StakingQuery`
  and `WasmQuery` query requests ([#802]).
- cosmwasm-std: In staking query response types `Delegation`, `FullDelegation`
  and `Validator` the validator address fields were changed from `HumanAddr` to
  `String`. The new `Addr` type cannot be used here because it only supports
  standard account addresses via `Api::addr_*` ([#871]).
- cosmwasm-std: Change address types in `BankMsg`, `IbcMsg` and `WasmMsg` from
  `HumanAddr` to `String` ([#802]).
- cosmwasm-std: `Api::addr_humanize` now returns `Addr` instead of `HumanAddr`
  ([#802]).
- cosmwasm-std: Hide `StakingMsg`, `CosmosMsg::Staking`,
  `AllDelegationsResponse`, `BondedDenomResponse`, `Delegation`,
  `FullDelegation`, `StakingQuery`, `Validator`, `ValidatorsResponse` and
  `testing::StakingQuerier` behind the `staking` feature flag to make those only
  available in contracts built for PoS chains.
- cosmwasm-std: Remove `StakingMsg::Withdraw` in favour of
  `DistributionMsg::SetWithdrawAddress` and
  `DistributionMsg::WithdrawDelegatorReward` ([#848]).
- cosmwasm-std: Rename `StakingQuery::Validators`, `ValidatorsResponse` and
  `QuerierWrapper::query_validators` to `StakingQuery::AllValidators`,
  `AllValidatorsResponse` and `QuerierWrapper.query_all_validators`. Add
  `StakingQuery::Validator`, `ValidatorResponse` and
  `QuerierWrapper::query_validator` to allow querying a single validator.
  ([#879])
- cosmwasm-schema: Make first argument non-mutable in `export_schema_with_title`
  for consistency with `export_schema`.
- cosmwasm-std: The block time in `BlockInfo::time` is now a `Timestamp`.
  `BlockInfo::time_nanos` was removed.

[#696]: https://github.com/CosmWasm/cosmwasm/issues/696
[#697]: https://github.com/CosmWasm/cosmwasm/issues/697
[#736]: https://github.com/CosmWasm/cosmwasm/pull/736
[#690]: https://github.com/CosmWasm/cosmwasm/issues/690
[@yihuang]: https://github.com/yihuang
[#853]: https://github.com/CosmWasm/cosmwasm/pull/853
[#858]: https://github.com/CosmWasm/cosmwasm/issues/858
[u128]: https://doc.rust-lang.org/std/primitive.u128.html
[#802]: https://github.com/CosmWasm/cosmwasm/pull/802
[#871]: https://github.com/CosmWasm/cosmwasm/issues/871
[#861]: https://github.com/CosmWasm/cosmwasm/issues/861
[#848]: https://github.com/CosmWasm/cosmwasm/issues/848
[#879]: https://github.com/CosmWasm/cosmwasm/pull/879

### Deprecated

- cosmwasm-std: `InitResponse`, `MigrateResponse` and `HandleResponse` are
  deprecated in favour of the new `Response`.
- cosmwasm-std: `Context` is deprecated in favour of the new mutable helpers in
  `Response`.
- cosmwasm-std: `HumanAddr` is not much more than an alias to `String` and it
  does not provide significant safety advantages. With CosmWasm 0.14, we now use
  `String` when there was `HumanAddr` before. There is also the new `Addr`,
  which holds a validated immutable human readable address. ([#802])

[#802]: https://github.com/CosmWasm/cosmwasm/pull/802

## [0.13.2] - 2021-01-14

## Changed

- cosmwasm-vm: Update Wasmer to 1.0.1.

## [0.13.1] - 2021-01-12

### Added

- cosmwasm-std: Add the new `#[entry_point]` macro attribute that serves as an
  alternative implementation to `cosmwasm_std::create_entry_points!(contract)`
  and `cosmwasm_std::create_entry_points_with_migration!(contract)`. Both ways
  are supported in the 0.13 series.

## [0.13.0] – 2021-01-06

## Added

- cosmwasm-std: Extend binary to array support to 64 bytes.

## Changed

- all: Drop support for Rust versions lower than 1.47.0.
- cosmwasm-std: Remove `cosmwasm_std::testing::MockApi::new`. Use
  `MockApi::default` instead.
- cosmwasm-vm: Upgrade Wasmer to 1.0 and adapt all the internal workings
  accordingly.
- cosmwasm-vm: Export method `cosmwasm_vm::Cache::stats` and response type
  `Stats`.
- cosmwasm-vm: Remove `cosmwasm_vm::testing::MockApi::new`. Use
  `MockApi::default` instead.
- cosmwasm-vm: Convert field `Instance::api` to a method.
- cosmwasm-vm: Change order of generic arguments for consistency in `Instance`,
  `Cache` and `Backend` to always match `<A: Api, S: Storage, Q: Querier>`.
- cosmwasm-vm: Remove `Instance::get_memory_size`. Use `Instance::memory_pages`
  instead.

## 0.12.2 (2020-12-14)

**cosmwasm-std**

- `StdError` now implements `PartialEq` (ignoring backtrace if any). This allows
  simpler `assert_eq!()` when testing error conditions (rather than match
  statements as now).

## 0.12.1 (2020-12-09)

**cosmwasm-std**

- Deprecate `InitResult`, `HandleResult`, `MigrateResult` and `QueryResult` in
  order to make error type explicit and encourage migration to custom errors.
- Implement `Deref` for `QuerierWrapper`, such that `QuerierWrapper` behaves
  like a smart pointer to `Querier` allowing you to access `Querier` methods
  directly.

## 0.12.0 (2020-11-19)

**cosmwasm-std**

- Remove the previously deprecated `StdError::Unauthorized`. Contract specific
  errors should be implemented using custom error types now (see
  [migration guide](./MIGRATING.md) 0.10 -> 0.11).
- Use dependency `thiserror` instead of `snafu` to implement `StdError`. Along
  with this change, the `backtraces` feature now requires Rust nightly.
- Rename `StdError::ParseErr::source` to `StdError::ParseErr::source_type` and
  `StdError::SerializeErr::target` to `StdError::SerializeErr::target_type` to
  work around speacial treatment of the field name `source` in thiserror.
- Rename `Extern` to `Deps` to unify naming.
- Simplify ownership of calling `handle`, etc. with `Deps` and `DepsMut` struct
  that just contains references (`DepsMut` has `&mut Storage` otherwise the
  same)
- Remove unused `Deps::change_querier`. If you need this or similar
  functionality, create a new struct with the right querier.
- Remove `ReadonlyStorage`. You can just use `Storage` everywhere. And use
  `&Storage` to provide readonly access. This was only needed to let
  `PrefixedStorage`/`ReadonlyPrefixedStorage` implement the common interface,
  which is something we don't need.

**cosmwasm-storage**

- `PrefixedStorage`/`ReadonlyPrefixedStorage` do not implement the
  `Storage`/`ReadonlyStorage` traits anymore. If you need nested prefixes, you
  need to construct them directly via `PrefixedStorage::multilevel` and
  `ReadonlyPrefixedStorage::multilevel`.
- Remove unused `TypedStorage`. If you need this or similar functionality, you
  probably want to use `Bucket` or `Singleton`. If you really need it, please
  copy the v0.11 code into your project.
- Remove `StorageTransaction` along with `transactional` and `RepLog`. This has
  not been used actively for contract development and is now maintained in a
  contract testing framework.

**cosmwasm-vm**

- Remove `Storage::range` and `StorageIterator`. The storage implementation is
  now responsible for maintaining iterators internally and make them accessible
  via the new `Storage::scan` and `Storage::next` methods.
- Add `FfiError::IteratorDoesNotExist`. Looking at this, `FfiError` should
  probably be renamed to something that includes before, on and behind the FFI
  boundary to Go.
- `MockStorage` now implements the new `Storage` trait and has an additional
  `MockStorage::all` for getting all elements of an iterator in tests.
- Remove unused `Extern::change_querier`. If you need this or similar
  functionality, create a new struct with the right querier.
- Let `Instance::from_code` and `CosmCache::get_instance` take options as an
  `InstanceOptions` struct. This contains `gas_limit` and `print_debug` for now
  and can easily be extended. `cosmwasm_vm::testing::mock_instance_options` can
  be used for creating such a struct in integration tests.
- Make `FileSystemCache` crate internal. This should be used via `CosmCache`.
- Fix return type of `FileSystemCache::load` to `VmResult<Option<Module>>` in
  order to differentiate missing files from errors.
- Add in-memory caching for recently used Wasm modules.
- Rename `CosmCache` to just `cosmwasm_vm::Cache` and add `CacheOptions` to
  configure it.
- Rename `Extern` to `Backend`.
- Rename `mock_dependencies` to `mock_backend` and
  `mock_dependencies_with_balances` to `mock_backend_with_balances`.
- Rename `FfiError`/`FfiResult` to `BackendError`/`BackendResult` and adapt
  `VmError` accordingly.

## 0.11.2 (2020-10-26)

**cosmwasm-std**

- Implement `From<std::str::Utf8Error>` and `From<std::string::FromUtf8Error>`
  for `StdError`.
- Generalize denom argument from `&str` to `S: Into<String>` in `coin`, `coins`
  and `Coin::new`.
- Implement `PartialEq` between `Binary` and `Vec<u8>`/`&[u8]`.
- Add missing `PartialEq` implementations between `HumanAddr` and `str`/`&str`.
- Add `Binary::to_array`, which allows you to copy binary content into a
  fixed-length `u8` array. This is especially useful for creating integers from
  binary data.

## 0.11.1 (2020-10-12)

**cosmwasm-std**

- Implement `Hash` and `Eq` for `Binary` to allow using `Binary` in `HashSet`
  and `HashMap`.
- Implement `Hash` and `Eq` for `CanonicalAddr` to allow using `CanonicalAddr`
  in `HashSet` and `HashMap`.
- Implement `Add`, `AddAssign` and `Sub` with references on the right hand side
  for `Uint128`.
- Implement `Sum<Uint128>` and `Sum<&'a Uint128>` for `Uint128`.

## 0.11.0 (2020-10-08)

**all**

- Drop support for Rust versions lower than 1.45.2.
- The serialization of the result from `init`/`migrate`/`handle`/`query` changed
  in an incompatible way. See the new `ContractResult` and `SystemResult` types
  and their documentation.
- Pass `Env` into `query` as well. As this doesn't have `MessageInfo`, we
  removed `MessageInfo` from `Env` and pass that as a separate argument to
  `init`, `handle`, and `query`. See the example
  [type definitions in the README](README.md#implementing-the-smart-contract) to
  see how to update your contract exports (just add one extra arg each).

**cosmwasm-std**

- Add `time_nanos` to `BlockInfo` allowing access to high precision block times.
- Change `FullDelegation::accumulated_rewards` from `Coin` to `Vec<Coin>`.
- Rename `InitResponse::log`, `MigrateResponse::log` and `HandleResponse::log`
  to `InitResponse::attributes`, `MigrateResponse::attributes` and
  `HandleResponse::attributes`.
- Rename `LogAttribute` to `Attribute`.
- Rename `log` to `attr`.
- Rename `Context::add_log` to `Context::add_attribute`.
- Add `Api::debug` for emitting debug messages during development.
- Fix error type for response parsing errors in `ExternalQuerier::raw_query`.
  This was `Ok(Err(StdError::ParseErr))` instead of
  `Err(SystemError::InvalidResponse)`, implying an error created in the target
  contract.
- Deprecate `StdError::Unauthorized` and `StdError::unauthorized` in favour of
  custom errors. From now on `StdError` should only be created by the standard
  library and should only contain cases the standard library needs.
- Let `impl Display for CanonicalAddr` use upper case hex instead of base64.
  This also affects `CanonicalAddr::to_string`.
- Create trait `CustomQuery` for the generic argument in
  `QueryRequest<C: CustomQuery>`. This allows us to provide
  `impl<C: CustomQuery> From<C> for QueryRequest<C>` for any custom query.
- Implement `From<Binary> for Vec<u8>`.
- Implement `From<CanonicalAddr> for Vec<u8>`.
- Add `Binary::into_vec` and `CanonicalAddr::into_vec`.
- The `canonical_length` argument was removed from `mock_dependencies`,
  `mock_dependencies_with_balances`. In the now deprecated `MockApi::new`, the
  argument is unused. Contracts should not need to set this value and usually
  should not make assumptions about the value.
- The canonical address encoding in `MockApi::canonical_address` and
  `MockApi::human_address` was changed to an unpredictable representation of
  non-standard length that aims to destroy most of the input structure.

**cosmwasm-storage**

- Change order of arguments such that `storage` is always first followed by
  namespace in `Bucket::new`, `Bucket::multilevel`, `ReadonlyBucket::new`,
  `ReadonlyBucket::multilevel`, `bucket` and `bucket_read`.
- Change order of arguments such that `storage` is always first followed by
  namespace in `PrefixedStorage::new`, `PrefixedStorage::multilevel`,
  `ReadonlyPrefixedStorage::new`, `ReadonlyPrefixedStorage::multilevel`,
  `prefixed` and `prefixed_read`.

**cosmwasm-vm**

- `CosmCache::new`, `Instance::from_code` and `Instance::from_module` now take
  an additional argument to enable/disable printing debug logs from contracts.
- Bump required export `cosmwasm_vm_version_3` to `cosmwasm_vm_version_4`.
- The `canonical_length` argument was removed from `mock_dependencies`,
  `mock_dependencies_with_balances` and `MockApi::new_failing`. In the now
  deprecated `MockApi::new`, the argument is unused. Contracts should not need
  to set this value and usually should not make assumptions about the value.
- The canonical address encoding in `MockApi::canonical_address` and
  `MockApi::human_address` was changed to an unpredictable representation of
  non-standard length that aims to destroy most of the input structure.

## 0.10.1 (2020-08-25)

**cosmwasm-std**

- Fix bug where `ExternalStorage.range()` would cause VM error if either lower
  or upper bound was set
  ([#508](https://github.com/CosmWasm/cosmwasm/issues/508))

## 0.10.0 (2020-07-30)

**all**

- Drop support for Rust versions lower than 1.44.1.

**cosmwasm-std**

- Remove error helpers `generic_err`, `invalid_base64`, `invalid_utf8`,
  `not_found`, `parse_err`, `serialize_err`, `underflow`, `unauthorized` in
  favour of `StdError::generic_err` and friends.
- Implement `From<&[u8; $N]> for Binary` and `From<[u8; $N]> for Binary` for all
  `$N <= 32`.
- Add `Context` object that can be used to build Init/Handle/Migrate response
  via `add_log`, `add_message`, `set_data` and then convert to the proper type
  via `into` or `try_into`. Option to simplify response construction.
- Env uses `HumanAddr` for `message.sender` and `contract_address`
- Remove `Api` argument from `mock_env`
- Implement `From<&[u8]>` and `From<Vec<u8>>` for `CanonicalAddr`

**cosmwasm-vm**

- Remove unused cache size argument from `CosmCache`.
- `set_gas_limit` now panics if the given gas limit exceeds the max. supported
  value.
- Increase the max. supported value for gas limit from 10_000_000_000 to
  0x7FFFFFFFFFFFFFFF.
- Add checks to `get_region` for failing early when the contract sends a Region
  pointer to the VM that is not backed by a plausible Region. This helps
  development of standard libraries.
- Create dedicated `RegionValidationError` and `RegionValidationResult`.
- `Api::human_address` and `Api::canonical_address` now return a pair of return
  data and gas usage.
- Remove `NextItem` in favour of a more advanced `FfiResult<T>`, which is used
  to store the return result and the gas information consistently across all
  APIs. `FfiResult<T>` was changed to `(Result<T, FfiError>, GasInfo)`.
- Create error type `FfiError::InvalidUtf8` for the cases where the backend
  sends invalid UTF-8 in places that expect strings.
- Remove `FfiError::Other` in favour of `FfiError::UserErr` and
  `FfiError::Unknown`.
- The `canonicalize_address` and `humanize_address` imports now report user
  errors to the contract.
- Bump `cosmwasm_vm_version_2` to `cosmwasm_vm_version_3`.
- `Querier::raw_query` and `QuerierResult` were removed in favour of the new
  `Querier::query_raw`, which includes a gas limit parameter for the query.

## 0.9.4 (2020-07-16)

**cosmwasm-vm**

- Add `Instance::create_gas_report` returning a gas report including the
  original limit, the remaining gas and the internally/externally used gas.

## 0.9.3 (2020-07-08)

**cosmwasm-storage**

- Add `.remove()` method to `Bucket` and `Singleton`.

## 0.9.2 (2020-06-29)

- Downgrade wasmer to 0.17.0.

## 0.9.1 (2020-06-25)

**cosmwasm-std**

- Replace type `Never` with `Empty` because enums with no cases cannot be
  expressed in valid JSON Schema.

## 0.9.0 (2020-06-25)

Note: this version contains an API bug and should not be used (see
https://github.com/CosmWasm/cosmwasm/issues/451).

**all**

- Upgrade wasmer to 0.17.1.
- Drop support for Rust versions lower than 1.43.1

**cosmwasm-std**

- `ReadonlyStorage::get` and all its implementations now return
  `Option<Vec<u8>>`.
- `ReadonlyStorage::range` and all its implementations now always succeed and
  return an iterator instead of a result. This is now an iterator over
  `Option<KV>` instead of `Option<StdResult<KV>>`.
- `Storage::{set, remove}` and all their implementations no longer have a return
  value. Previously they returned `StdResult<()>`.
- Trait `Querier` is not `Clone` and `Send` anymore.
- `consume_region` panics on null pointers and returns `Vec<u8>` instead of
  `StdResult<Vec<u8>>`.
- Added contract migration mechanism. Contracts can now optionally export a
  `migrate` function with the following definition:
  ```rust
  extern "C" fn migrate(env_ptr: u32, msg_ptr: u32) -> u32;
  ```
- InitResponse no longer has a data field. We always return the contract address
  in the data field in the blockchain and don't allow you to override. `handle`
  can still make use of the field.
- Rename `MockQuerier::with_staking` to `MockQuerier::update_staking` to match
  `::update_balance`.
- The obsolete `StdError::NullPointer` and `null_pointer` were removed.
- Error creator functions are now in type itself, e.g.
  `StdError::invalid_base64` instead of `invalid_base64`. The free functions are
  deprecated and will be removed before 1.0.

**cosmwasm-storage**

- Remove `transactional_deps`. Use `transactional` that just provides a
  transactional storage instead.
- `get_with_prefix` returns `Option<Vec<u8>>` instead of
  `StdResult<Option<Vec<u8>>>`.
- `set_with_prefix` and `remove_with_prefix` return nothing instead of
  `StdResult<()>`.
- `RepLog::commit` no longer returns any value (always succeeds).
- `Op::apply` no longer returns any value (always succeeds).

**cosmwasm-vm**

- The export `allocate` must not return 0 as a valid address. The contract is
  responsible for avoiding this offset in the linear memory.
- The import `db_read` now allocates memory for the return value as part of the
  call and returns a pointer to the value as `u32`. The return value 0 means
  _key does not exist_.
- The import `db_next` now allocates a memory region for the return key and
  value as part of the call and returns a pointer to the region as `u32`. The
  data in the region is stored in the format `value || key || keylen`. As
  before, an empty key means _no more value_.
- Remove `Instance::get_gas` in favour of `Instance::get_gas_left`.
- All calls from the VM layer to the chain layer also return the amount of gas
  used on success. (This is represented by replacing the return value with
  `(value, used_gas)`). Gas usage across the system is then tracked in the VM
  layer, which allows us to halt the contract during an import, as soon as we
  can prove that we used all allocated gas.
- Remove instance caching, which is disabled since 0.8.1 as it is not stable.
  Remove `CosmCache::store_instance`; you can not call `Instance::recylce`
  directly to get back the external dependencies.
- Rename `MockQuerier::with_staking` to `MockQuerier::update_staking` to match
  `::update_balance`.
- Instead of panicking, `read_region`/`write_region`/`get_region`/`set_region`
  now return a new `CommunicationError::DerefErr` when dereferencing a pointer
  provided by the contract fails.
- `FfiError::set_message` was removed because errors should be immutable. Use
  `FfiError::other` to create an error with the desired error message.
- The import implementation of `db_scan` now errors instead of returning an
  error code for an invalid order value. The return type was changed to `u32`.
- Remove `StorageIteratorItem` in favour of the new types `StorageIterator` and
  `NextItem`. `StorageIterator` is a custom iterator type that does not
  implement Rust's `Iterator` trait, allowing it to communicate the used gas
  value of the last `next` call to the VM.
- Don't report any `VmError` back to the contract in `canonicalize_address` and
  `humanize_address`. Only invalid inputs should be reported.
- Move error cases `VmError::RegionLengthTooBig` and `VmError::RegionTooSmall`
  into `CommunicationError`.
- In the `canonicalize_address` implementation, invalid UTF-8 inputs now result
  in `CommunicationError::InvalidUtf8`, which is not reported back to the
  contract. A standard library should ensure this never happens by correctly
  encoding string input values.
- Merge trait `ReadonlyStorage` into `Storage`.
- The imports `canonicalize_address` and `humanize_address` now return a memory
  address to an error `Region`. If this address is 0, the call succeeded.
- Bump `cosmwasm_vm_version_1` to `cosmwasm_vm_version_2`.

## 0.8.1 (2020-06-08)

**cosmwasm-std**

- The arguments of `log` changed from `&str` to `ToString`, allowing to pass
  various types like `String`, `HumanAddr`, `Uint128` or primitive integers
  directly.
- Add `From<Vec<u8>>` and `Into<Vec<u8>>` implementations for `Binary` for
  zero-copy conversions.

**cosmwasm-vm**

- Deprecated `Instance::get_gas` in favour of `Instance::get_gas_left`. The old
  method will remain available for a while but will issue a deprecation warning
  when used.
- Disable instance caching by treating every cache size as 0. Instance caching
  is not safe as the same Wasm memory is reused across multiple executions.
- The storage of an `Instance` can now be set into readonly mode, which is
  checked by the writing storage imports `db_write` and `db_remove`. Read-only
  mode is off by default for backwards compatibility. `call_query_raw` now sets
  the instance's storage to readonly.
- The new error case `VmError::WriteAccessDenied` is returned when a contract
  calls an import that potentially writes to storage during a query.

## 0.8.0 (2020-05-25)

**all**

- Upgrade schemars to 0.7.
- Upgrade wasmer to 0.17.
- Update snafu to 0.6.
- Minimal supported Rust version is 1.41.
- Split `Region.len` into `Region.capacity` and `Region.length`, where the new
  capacity is the number of bytes available and `length` is the number of bytes
  used. This is a breaking change in the contract-vm interface, which requires
  the same memory layout of the `Region` struct on both sides.
- Add `remove` method to `Storage` trait.
- (feature-flagged) Add `range` method to `ReadonlyStorage` trait. This returns
  an iterator that covers all or a subset of the items in the db ordered
  ascending or descending by key.
- Add new feature flag `iterator` to both packages to enable `range`
  functionality. This is used to allow potential porting to chains that use
  Merkle Tries (which don't allow iterating over ranges).
- All serialized JSON types now use snake_case mappings for names. This means
  enum fields like `ChangeOwner` will map to `change_owner` in the underlying
  JSON, not `changeowner`. This is a breaking change for the clients.
- Public interface between contract and runtime no longer uses `String` to
  represent an error, but rather serializes `ApiError` as a rich JSON error.
- Return value from `env.write_db` and `env.remove_db` to allow error reporting.
- Query responses are now required to contain valid JSON.
- Renamed all `*_db` wasm imports to `db_*`
- Merge `cw-storage` repo as subpackage, now `cosmwasm-storage`
- Add iterator support to `cosmwasm-storage`
- `Coin.amount` is now `Uint128` rather than `String`. Uint128 serializes as a
  string in JSON, but parses into a u128 data in memory. It also has some
  operator overloads to allow easy math operations on `Coin` types, as well as
  enforcing valid amounts.
- `Env` no longer has a `contract.balance` element. If you need this info,
  please use the `Querier` to get this info. As of Cosmos-SDK 0.39 this needs
  extra storage queries to get the balance, so we only do those queries when
  needed.
- `Env.message.sent_funds` is a `Vec<Coin>` not `Option<Vec<Coin>>`. We will
  normalize the go response in `go-cosmwasm` before sending it to the contract.
- `Env.message.signer` was renamed to `Env.message.sender`.
- `Env.block.{height,time}` are now `u64` rather than `i64`.

**cosmwasm-schema**

- This new crate now contains the implementations for generating JSON Schema
  files from interface types. It exposes the functions `export_schema`,
  `export_schema_with_title`, and `schema_for`.

**cosmwasm-std**

- Make all symbols from `cosmwasm::memory` crate internal, as those symbols are
  not needed by users of the library.
- Rename `cosmwasm::mock::dependencies` -> `cosmwasm::mock::mock_dependencies`
  to differentiate between testing and production `External`.
- Export all symbols from `cosmwasm::mock` as the new non-wasm32 module
  `cosmwasm::testing`. Export all remaining symbols at top level (e.g.
  `use cosmwasm::traits::{Api, Storage};` + `use cosmwasm::encoding::Binary;`
  becomes `use cosmwasm::{Api, Binary, Storage};`).
- Rename package `cosmwasm` to `cosmwasm-std`.
- The export `allocate` does not zero-fill the allocated memory anymore.
- Add `remove_db` to the required imports of a contract.
- (feature-flagged) add `scan_db` and `next_db` callbacks from wasm contract to
  VM.
- `serde::{from_slice, to_vec}` return `cosmwasm_std::Result`, no more need to
  use `.context(...)` when calling these functions
- Split `Response` into `InitResponse` and `HandleResponse`; split
  `ContractResult` into `InitResult` and `HandleResult`.
- Create explicit `QueryResponse`, analogue to `InitResponse` and
  `HandleResponse`.
- The exports `cosmwasm_vm_version_1`, `allocate` and `deallocate` are now
  private and can only be called via the Wasm export. Make sure to `use`
  `cosmwasm_std` at least once in the contract to pull in the C exports.
- Add `Querier` trait and `QueryRequest` for query callbacks from the contract,
  along with `SystemError` type for the runtime rejecting messages.
- `QueryRequest` takes a generic `Custom(T)` type that is passed opaquely to the
  end consumer (`wasmd` or integration test stubs), allowing custom queries to
  native code.
- `{Init,Handle,Query}Result` are now just aliases for a concrete `ApiResult`
  type.
- Support results up to 128 KiB in `ExternalStorage.get`.
- The `Storage` trait's `.get`, `.set` and `.remove` now return a `Result` to
  allow propagation of errors.
- Move `transactional`, `transactional_deps`, `RepLog`, `StorageTransaction`
  into crate `cosmwasm-storage`.
- Rename `Result` to `StdResult` to differentiate between the auto-`use`d
  `core::result::Result`. Fix error argument to `Error`.
- Complete overhaul of `Error` into `StdError`:
  - The `StdError` enum can now be serialized and deserialized (losing its
    backtrace), which allows us to pass them over the Wasm/VM boundary. This
    allows using fully structured errors in e.g. integration tests.
  - Auto generated snafu error constructor structs like `NotFound`/`ParseErr`/…
    have been intenalized in favour of error generation helpers like
    `not_found`/`parse_err`/…
  - All error generator functions now return errors instead of results.
  - Error cases don't contain `source` fields anymore. Instead source errors are
    converted to standard types like `String`. For this reason, both
    `snafu::ResultExt` and `snafu::OptionExt` cannot be used anymore.
  - Backtraces became optional. Use `RUST_BACKTRACE=1` to enable them.
  - `Utf8Err`/`Utf8StringErr` merged into `StdError::InvalidUtf8`
  - `Base64Err` renamed into `StdError::InvalidBase64`
  - `ContractErr`/`DynContractErr` merged into `StdError::GeneralErr`
  - The unused `ValidationErr` was removed
  - `StdError` is now
    [non_exhaustive](https://doc.rust-lang.org/1.35.0/unstable-book/language-features/non-exhaustive.html),
    making new error cases non-breaking changes.
- `ExternalStorage.get` now returns an empty vector if a storage entry exists
  but has an empty value. Before, this was normalized to `None`.
- Reorganize `CosmosMsg` enum types. They are now split by modules:
  `CosmosMsg::Bank(BankMsg)`, `CosmosMsg::Custom(T)`, `CosmosMsg::Wasm(WasmMsg)`
- CosmosMsg is now generic over the content of `Custom` variant. This allows
  blockchains to support custom native calls in their Cosmos-SDK apps and
  developers to make use of them in CosmWasm apps without forking the
  `cosmwasm-vm` and `go-cosmwasm` runtime.
- Add `staking` feature flag to expose new `StakingMsg` types under `CosmosMsg`
  and new `StakingRequest` types under `QueryRequest`.
- Add support for mocking-out staking queries via `MockQuerier.with_staking`
- `from_slice`/`from_binary` now require result type to be `DeserializeOwned`,
  i.e. the result must not contain references such as `&str`.

**cosmwasm-vm**

- Make `Instance.memory`/`.allocate`/`.deallocate`/`.func` crate internal. A
  user of the VM must not access the instance's memory directly.
- The imports `env.canonicalize_address`, `env.humanize_address` and
  `env.read_db` don't return the number of bytes written anymore. This value is
  now available in the resulting regions. Negative return values are errors, 0
  is success and values greater than 0 are reserved for future use.
- Change the required interface version guard export from `cosmwasm_api_0_6` to
  `cosmwasm_vm_version_1`.
- Provide implementations for `remove_db` and (feature-flagged) `scan_db` and
  `next_db`
- Provide custom `serde::{from_slice, to_vec}` implementation separate from
  `cosmwasm_std`, so we can return cosmwasm-vm specific `Result` (only used
  internally).
- `call_{init,handle,query}` and the `cosmwasm_vm::testing` wrappers return
  standard `Result` types now, eg. `Result<HandleResponse, ApiError>`.
- Add length limit when reading memory from the instance to protect against
  malicious contracts creating overly large `Region`s.
- Add `Instance.get_memory_size`, giving you the peak memory consumption of an
  instance.
- Remove `cosmwasm_vm::errors::CacheExt`.
- Move `cosmwasm_vm::errors::{Error, Result}` to
  `cosmwasm_vm::{VmError, VmResult}` and remove generic error type from result.
- The import `db_read` now returns an error code if the storage key does not
  exist. The latest standard library converts this error code back to a `None`
  value. This allows differentiating non-existent and empty storage entries.
- Make `Instance::from_module`, `::from_wasmer` and `::recycle` crate-internal.
- Create explicit, public `Checksum` type to identify Wasm blobs.
- `CosmCache::new` now takes supported features as an argument.
- Rename `VmError::RegionTooSmallErr` to `VmError::RegionTooSmall`.
- Rename `VmError::RegionLengthTooBigErr` to `VmError::RegionLengthTooBig`.
- Change property types to owned string in `VmError::UninitializedContextData`,
  `VmError::ConversionErr`, `VmError::ParseErr` and `VmError::SerializeErr`.
- Remove `VmError::IoErr` in favour of `VmError::CacheErr`.
- Simplify `VmError::CompileErr`, `VmError::ResolveErr` and
  `VmError::WasmerRuntimeErr` to just hold a string with the details instead of
  the source error.
- Remove `VmError::WasmerErr` in favour of the new `VmError::InstantiationErr`.
- The snafu error builders from `VmError` are now private, i.e. callers can only
  use the errors, not create them.
- `VmError` is now `#[non_exhaustive]`.
- Split `VmError::RuntimeErr` in `VmError::BackendErr` and
  `VmError::GenericErr`; rename `VmError::WasmerRuntimeErr` to
  `VmError::RuntimeErr`.
- Add `Instance.with_querier` analogue to `Instance.with_storage`.

## 0.7.2 (2020-03-23)

**cosmwasm**

- Fix JSON schema type of `Binary` from int array (wrong) to string (right).
- Make `Extern` not `Clone`able anymore. Before cloning led to copying the data
  for mock storage and copying a stateless bridge for the external storage,
  which are different semantics.
- Remove public `cosmwasm::imports::dependencies`. A user of this library does
  not need to call this explicitely. Dependencies are created internally and
  passed as an argument in `exports::do_init`, `exports::do_handle` and
  `exports::do_query`.
- Make `ExternalStorage` not `Clone`able anymore. This does not copy any data,
  so a clone could lead to unexpected results.

## 0.7.1 (2020-03-11)

**cosmwasm_vm**

- Avoid unnecessary panic when checking corrupted wasm file.
- Support saving the same wasm to cache multiple times.

## 0.7.0 (2020-02-26)

**cosmwasm**

- Rename `Slice` to `Region` to simplify differentiation between Wasm memory
  region and serde's `from_slice`
- Rename `Params` to `Env`, `mock_params` to `mock_env` for clearer naming (this
  is information on the execution environment)
- `Response.log` is not a vector of key/value pairs that can later be indexed by
  Tendermint.

**cosmwasm_vm**

- Remove export `cosmwasm_vm::read_memory`. Using this indicates an
  architectural flaw, since this is a method for host to guest communication
  inside the VM and not needed for users of the VM.
- Create new type `cosmwasm_vm:errors::Error::RegionTooSmallErr`.
- Change return type of `cosmwasm_vm::write_memory` to `Result<usize, Error>` to
  make it harder to forget handling errors.
- Fix missing error propagation in `do_canonical_address`, `do_human_address`
  and `allocate`.
- Update error return codes in import `c_read`.
- Rename imports `c_read`/`c_write` to `read_db`/`write_db`.
- Rename imports `c_canonical_address`/`c_human_address` to
  `canonicalize_address`/`humanize_address`.
- Add `cosmwasm_vm::testing::test_io` for basic memory allocation/deallocation
  testing between host and guest.
- Make `ValidationErr.msg` a dynamic `String` including relevant runtime
  information.
- Remove export `check_api_compatibility`. The VM will take care of calling it.
- Let `check_api_compatibility` check imports by fully qualified identifier
  `<module>.<name>`.
- Make gas limit immutable in `cosmwasm_vm::instance::Instance`. It is passed
  once at construction time and cannot publicly be manipulated anymore.
- Remove `take_storage`/`leave_storage` from `cosmwasm_vm::Instance`.

## 0.6

[Define canonical address callbacks](https://github.com/confio/cosmwasm/issues/73)

- Use `&[u8]` for addresses in params
- Allow contracts to resolve human readable addresses (`&str`) in their json
  into a fixed-size binary representation
- Provide mocks for unit testing and integration tests

- Separate out `Storage` from `ReadOnlyStorage` as separate traits

## 0.5

### 0.5.2

This is the first documented and supported implementation. It contains the basic
feature set. `init` and `handle` supported for modules and can return messages.
A stub implementation of `query` is done, which is likely to be deprecated soon.
Some main points:

- The build-system and unit/integration-test setup is all stabilized.
- Cosmwasm-vm supports singlepass and cranelift backends, and caches modules on
  disk and instances in memory (lru cache).
- JSON Schema output works

All future Changelog entries will reference this base

[unreleased]: https://github.com/CosmWasm/cosmwasm/compare/v1.2.3...HEAD
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
[1.0.0-rc.0]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta8...v1.0.0-rc.0
[1.0.0-beta8]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta7...v1.0.0-beta8
[1.0.0-beta7]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta6...v1.0.0-beta7
[1.0.0-beta6]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta5...v1.0.0-beta6
[1.0.0-beta5]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta4...v1.0.0-beta5
[1.0.0-beta4]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta3...v1.0.0-beta4
[1.0.0-beta3]:
  https://github.com/CosmWasm/cosmwasm/compare/v1.0.0-beta...v1.0.0-beta3
[1.0.0-beta]: https://github.com/CosmWasm/cosmwasm/compare/v0.16.2...v1.0.0-beta
[0.16.2]: https://github.com/CosmWasm/cosmwasm/compare/v0.16.1...v0.16.2
[0.16.1]: https://github.com/CosmWasm/cosmwasm/compare/v0.16.0...v0.16.1
[0.16.0]: https://github.com/CosmWasm/cosmwasm/compare/v0.15.2...v0.16.0
[0.15.2]: https://github.com/CosmWasm/cosmwasm/compare/v0.15.1...v0.15.2
[0.15.1]: https://github.com/CosmWasm/cosmwasm/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/CosmWasm/cosmwasm/compare/v0.14.1...v0.15.0
[0.14.1]: https://github.com/CosmWasm/cosmwasm/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.1...v0.14.0
[0.13.2]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/CosmWasm/cosmwasm/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/CosmWasm/cosmwasm/compare/v0.12.0...v0.13.0
