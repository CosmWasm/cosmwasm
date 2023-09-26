# Floaty Contract

This contract contains all WebAssembly floating point instructions. It is used
for testing the floating point support.

In order to compile it, you need a nightly version of Rust and enable the
`nontrapping-fptoint` target-feature. This allows the usage of
[some more conversion instructions](https://github.com/WebAssembly/spec/blob/main/proposals/nontrapping-float-to-int-conversion/Overview.md).
To do this, run:

```sh
RUSTFLAGS="-C link-arg=-s -C target-feature=+nontrapping-fptoint" cargo wasm
```
