# Checks in GitHub workflows

## Packages

List of checks performed in `packages.yml` workflow:

| Package                    |    channel    | fmt | linter | build | tests | wasm | thumbv7em |
|----------------------------|:-------------:|:---:|:------:|:-----:|:-----:|:----:|:---------:|
| **cosmwasm-check**         | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-core**          | 1.82.0 stable |  +  |   +    |   +   |   +   |  +   |     +     |
| **cosmwasm-crypto**        | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cw-schema**              | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cw-schema-derive**       | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-derive**        | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-schema**        | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-schema-derive** | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-std**           | 1.82.0 stable |  +  |   +    |   +   |   +   |  +   |           |
| **cosmwasm-vm**            | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |
| **cosmwasm-vm-derive**     | 1.82.0 stable |  +  |   +    |   +   |   +   |      |           |

## Contracts

List of checks performed in `contracts.yml` workflow: 

| Contract             |    channel     | fmt | unit<br/>tests | wasm | linter | integration<br/>tests | schema | check<br/>released | check<br/>current |
|----------------------|:--------------:|:---:|:--------------:|:----:|:------:|:---------------------:|:------:|:------------------:|:-----------------:|
| **burner**           | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **crypto-verify**    | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **cyberpunk**        | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **empty**            | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **floaty**           | 1.82.0 nightly |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **hackatom**         | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **ibc2**             | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **ibc-callbacks**    | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **ibc-reflect**      | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **ibc-reflect-send** | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **nested-contracts** | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **queue**            | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **reflect**          | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **replier**          | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **staking**          | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **virus**            | 1.82.0 stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |

Legend:

- **channel** - Rust channel used to run the checks. Possible values are **stable** or **nightly**.
  Most of the contracts use **stable** channel, but in some cases **nightly** channel is required.
- **fmt** - Checks code formatting against Rust formatting rules.
- **unit tests** - Runs all unit tests provided by the contract.
- **wasm** - Checks building the WASM binary for the contract.
- **linter** - Check the code correctness against Rust clippy rules.
- **integration tests** - Runs all integration tests provided by the contract.
- **schema** - Checks if there are no changes in contract interface (schema).
- **check released** - Checks the WASM binary using recently released version of `cosmwasm-check` tool.  
- **check current** - Checks the WASM binary using currently developed version of `cosmwasm-check` tool.

> All checks are executed on standard [GitHub runner images](https://github.com/actions/runner-images): 
> `ubuntu-latest` (x86_64), `macos-latest` (arm64) and `windows-latest` (x86_64).
