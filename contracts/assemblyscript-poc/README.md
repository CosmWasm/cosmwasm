# AssemblyScript smart contract (POC)

This is a proof of concept folder showing that it is possible to build smart
contracts in other laguages than Rust and what needs to be done. A secondary
motivation is to challence design decisions and ensure we have clean
cross-language APIs.

The motivation for adding AssemblyScript to CosmWasm can be found here: TODO:
INSERT LINK

## Dirctory structure

- `assemblyscript-poc` this project
  - `contract` the project that generated the wasm file
  - `tests` integration tests written in Rust utilizing the existing testing
    infrastructure

## Building

The following command shows you how to build and test the project.

```sh
(cd contract && yarn install && yarn build) && \
  (cd tests && cargo integration-test)
```

## License

This proof of concept project is licensed under
[(WTFPL OR 0BSD OR Unlicense OR MIT OR BSD-3-Clause)](https://spdx.org/licenses/).
Please note that this does not hold for CosmWasm as a whole nor for this
project's dependencies.
