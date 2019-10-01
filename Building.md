# Building WebAssembly Smart Contracts

The subdirectories are various examples of compiling smart contracts.
Here are some tips useful for creating your own.

## Setup

This needs to be compiled as a c dynamic lib. To do so, first generate the crate
via `cargo new --lib sample`. Then add the following to `Cargo.toml`:

```yaml
[lib]
crate-type = ["cdylib", "rlib"]
```

The `cdylib` is needed for the wasm target. 
The `rlib` is needed to compile artifacts for benchmarking (and integration tests).

**Note** throughout this demo I will use the name `sample` for the project. 
Please replace it with the real name of your crate. I intentionally didn't use <name>,
so you can cut and paste for a quick demo. Just update this when making real code.

## Requirements

You must support the rust target `wasm32-unknown-unknown`.

Check which ones you currently have installed via `rustup target list --installed`.
If wasm32 is not on that list, install using `rustup target add wasm32-unknown-unknown`


## Building

Go into the subdirectory, called `sample` from now on:

To compile the code, run  `cargo build --release --target wasm32-unknown-unknown`. 
The output will be in `target/wasm32-unknown-unknown/release/sample.wasm`

You probably don't want to explicitly set the target every time, so you can just
add the following to `.cargo/config`:

```yaml
[alias]
wasm = "build --release --target wasm32-unknown-unknown"
```

And you can now just call `cargo wasm` to build it, and `cargo test` to run tests.

**Note** Using `build.target` seems to force tests to use that target as well, remove this or find a work-around.
[This discussion](https://internals.rust-lang.org/t/set-default-target-for-cargo-build-but-not-for-cargo-test/9777)
and [closed PR](https://github.com/rust-lang/cargo/pull/6825) seem to suggest this will never be done.

## Optimizations

The size of the wasm output is critical if it is supposed to go on a blockchain.
Here are some things to make it smaller.

### Smaller builds

If you want to request the compiler to make smaller binaries, 
you can hit a few flags (which raise compile time significantly).
Try adding this custom profile to Cargo.toml:

```yaml
[profile.release]
opt-level = 3
debug = false
rpath = false
lto = true
debug-assertions = false
codegen-units = 1
panic = 'unwind'
incremental = false
overflow-checks = true
```

**IMPORTANT** it is essential that codegen-units is set to 1 for deterministic builds. 
Otherwise, you will have a different wasm output with each compile.

## Shrinking the output

After compiling your contract, take a look at the size of the original output:
`du -sh  target/wasm32-unknown-unknown/release/sample.wasm`, it is likely around 1 MB.
Most of that is unneeded and can easily be trimmed with `wasm-gc`.

```shell script
cargo wasm
cp target/wasm32-unknown-unknown/release/sample.wasm contact.wasm
du -h contract.wasm

cargo install wasm-gc
wasm-gc contract.wasm
du -h contract.wasm
```

A bit smaller, huh?

**TODO**: pack up these tools in a fixed Dockerfile so we have a
consistent setup to help with reproducible builds. 