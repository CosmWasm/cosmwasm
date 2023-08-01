# JsonSchema Go Type Generator

This is an internal utility to generate Go types from `cosmwasm-std`'s query
response types. These types can then be used in
[wasmvm](https://github.com/CosmWasm/wasmvm).

## Limitations

Only basic structs and enums are supported. Tuples don't work for example.

## License

This package is part of the cosmwasm repository, licensed under the Apache
License 2.0 (see [NOTICE](https://github.com/CosmWasm/cosmwasm/blob/main/NOTICE)
and [LICENSE](https://github.com/CosmWasm/cosmwasm/blob/main/LICENSE)).
