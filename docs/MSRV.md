# Minimum Supported Rust Version (MSRV)

This repository has two independent MSRVs, one for the standard library that is
compiled into contracts (cosmwasm-std MSRV) and one for the virtual machine
executing contracts (cosmwasm-vm MSRV). The other packages belong to one of the
two:

| Crate            | MSRV              |
| ---------------- | ----------------- |
| cosmwasm-crypto  | cosmwasm-std MSRV |
| cosmwasm-derive  | cosmwasm-std MSRV |
| cosmwasm-schema  | cosmwasm-std MSRV |
| cosmwasm-std     | cosmwasm-std MSRV |
| cosmwasm-storage | cosmwasm-std MSRV |
| cosmwasm-vm      | cosmwasm-vm MSRV  |

The reason for this is that cosmwasm-std has a wider audience than cosmwasm-vm
and we try to change the MSRV less frequently, allowing contract developers to
pick their favourite compiler version without getting disrupted. Another reason
is that cosmwasm-vm depends on [Wasmer], which bump their MSRV frequently.
Please note that as soon as you start using integration tests for contract
development, you will depends on the cosmwasm-vm MSRV.

[wasmer]: https://github.com/wasmerio/wasmer

## Latest changes

| Version | cosmwasm-std MSRV | cosmwasm-vm MSRV | Notes                                                                                   |
| ------- | ----------------- | ---------------- | --------------------------------------------------------------------------------------- |
| 1.0.0   | 1.53.0            | 1.53.0           | Not strictly needed but prepares for [Wasmer > 2] and let's us keep up with modern Rust |
| 0.14.0  | 1.51.0            | 1.51.0           | Added support for const generics                                                        |
| 0.13.2  | 1.47.0            | 1.48.0           | Through [Wasmer 1.0.1]                                                                  |
| 0.13.0  | 1.47.0            | 1.47.0           |                                                                                         |
| 0.11.0  | 1.45.2            | 1.45.2           |                                                                                         |

[wasmer 1.0.1]:
  https://github.com/wasmerio/wasmer/blob/master/CHANGELOG.md#101---2021-01-12
[wasmer > 2]:
  https://github.com/wasmerio/wasmer/commit/005d1295297acaaa7fdf713e76a36d08264d8c49

## Policy

**cosmwasm-std MSRV**

- It must always be at least one minor version behind latest stable. E.g. with
  stable Rust 1.33.3 it must not exceed 1.32.0.
- It can be bumped without a semver major release of the crates. However, a
  minor version bump is required.

**cosmwasm-vm MSRV**

- It can be bumped without a semver major release of the crate. However, a minor
  version bump is required.
- It is always higher or equal to cosmwasm-std MSRV because the VM depends on
  cosmwasm-std and related packages.
