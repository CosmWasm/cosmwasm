# Generate contract template

This is a quick way to set up a new contract using the cosmwasm framework.

## Requirements

First, make sure you have a recent version of rust installed (say 1.36+, we test on 1.37),
and the following requirements:

```shell script
rustup target add wasm32-unknown-unknown
cargo install wasm-gc
cargo install cargo-generate --features vendored-openssl
```

TODO: set up a repo for this

```yaml
[package]
name = "{{project-name}}"
version = "0.1.0"
authors = ["{{authors}}"]
edition = "2018"
```

Then run it:

`cargo generate --name foobar --git https://github.com/confio/cosmwasm-template.git`


Look at https://github.com/rustwasm/wasm-pack-template

And also how to use wasm-bindgen to get small wasm output. Some info:

* https://rustwasm.github.io/book/print.html
* https://rustwasm.github.io/book/reference/tools.html