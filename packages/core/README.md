# cosmwasm-core

[![cosmwasm-core on crates.io](https://img.shields.io/crates/v/cosmwasm-core.svg)](https://crates.io/crates/cosmwasm-core)

This crate contains components of cosmwasm-std that can be used in a
[no_std environment](https://docs.rust-embedded.org/book/intro/no-std.html). All
symbols are re-exported by cosmwasm-std, such that contract developers don't
need to add this dependency directly. It is recommended to only use cosmwasm-std
whenever possible.

## License

This package is part of the cosmwasm repository, licensed under the Apache
License 2.0 (see [NOTICE](https://github.com/CosmWasm/cosmwasm/blob/main/NOTICE)
and [LICENSE](https://github.com/CosmWasm/cosmwasm/blob/main/LICENSE)).
