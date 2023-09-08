# Floaty Contract

This contract contains all WebAssembly floating point instructions. It is used
for testing the floating point support.

In order to compile it, you need a nightly version of Rust and enable the
`nontrapping-fptoint` target-feature like this:

```sh
RUSTFLAGS="-C target-feature=+nontrapping-fptoint" cargo wasm
```
