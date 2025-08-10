# Checks in GitHub workflows

## Contracts

| Name              | channel | fmt | unit<br/>tests | wasm | linter | integration<br/>tests | schema | check<br/>released | check<br/>current |
|-------------------|:-------:|:---:|:--------------:|:----:|:------:|:---------------------:|:------:|:------------------:|:-----------------:|
| **burner**        | stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |
| **crypto-verify** | stable  |  +  |       +        |  +   |   +    |           +           |   +    |         +          |         +         |

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
