# Building WebAssembly Smart Contracts

The subdirectories are various examples of compiling smart contracts. Here are
some tips useful for creating your own.

## Setup

This needs to be compiled as a c dynamic lib. To do so, first generate the crate
via `cargo new --lib sample`. Then add the following to `Cargo.toml`:

```yaml
[lib]
crate-type = ["cdylib", "rlib"]
```

The `cdylib` is needed for the wasm target. The `rlib` is needed to compile
artifacts for integration tests (and benchmarking).

**Note** throughout this demo I will use the name `hackatom` for the project.
Please replace it with the real name of your crate. I intentionally didn't use
`<name>`, so you can cut and paste for a quick demo. Just update this when
making real code.

## Requirements

You must support the rust target `wasm32-unknown-unknown`.

Check which ones you currently have installed via
`rustup target list --installed`. If wasm32 is not on that list, install using
`rustup target add wasm32-unknown-unknown`

## Building

Go into the subdirectory, called `sample` from now on:

To compile the code, run
`cargo build --release --target wasm32-unknown-unknown`. The output will be in
`target/wasm32-unknown-unknown/release/hackatom.wasm`

You probably don't want to explicitly set the target every time, so you can just
add the following to `.cargo/config`:

```yaml
[alias]
wasm = "build --release --target wasm32-unknown-unknown"
```

And you can now just call `cargo wasm` to build it, and `cargo test` to run
tests.

**Note** Using `build.target` seems to force tests to use that target as well,
remove this or find a work-around.
[This discussion](https://internals.rust-lang.org/t/set-default-target-for-cargo-build-but-not-for-cargo-test/9777)
and [closed PR](https://github.com/rust-lang/cargo/pull/6825) seem to suggest
this will never be done.

## Optimizations

The size of the wasm output is critical if it is supposed to go on a blockchain.
Here are some things to make it smaller.

### Smaller builds

If you want to request the compiler to make smaller binaries, you can hit a few
flags (which raise compile time significantly). Try adding this custom profile
to Cargo.toml:

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

**IMPORTANT** it is essential that codegen-units is set to 1 for deterministic
builds. Otherwise, you will have a different wasm output with each compile.

## Shrinking the output

After compiling your contract, take a look at the size of the original output:
`du -sh target/wasm32-unknown-unknown/release/hackatom.wasm`, it is likely
around 1.5 MB. Most of that is unneeded and can easily be trimmed. The first
approach is to use [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to build
it. This is designed for exporting small wasm builds and js bindings for the
web, but is also the most actively maintained stack for trimmed wasm builds with
rust. [`wasm-gc`](https://github.com/alexcrichton/wasm-gc), the older
alternative, has been deprecated for this approach. (Note you must
[install wasm-pack first](https://rustwasm.github.io/wasm-pack/installer/)):

```sh
cargo wasm
du -h target/wasm32-unknown-unknown/release/hackatom.wasm

wasm-pack build
du -h pkg/hackatom_bg.wasm
```

A bit smaller, huh? For the sample contract, this is around 64kB, but this
varies a lot contract-by-contract. If you have plenty of dependencies and this
is still too big, you can do a bit of investigation of where the size comes
from, and maybe change your dependencies:

```sh
cargo install twiggy
twiggy top contract.wasm | head -20
twiggy garbage contract.wasm
twiggy dominators contract.wasm | less
```

Look at the imports and exports:

```sh
cargo install wasm-nm
wasm-nm -e contract.wasm
wasm-nm -i contract.wasm
```

## Ultra-Compression

You can still get a bit smaller. Note the symbol names that were used in twiggy.
Well, those come from inside the wasm build. You can strip out these symbols and
other debug info, that won't be usable when running anyway. For this we use
`wasm-opt` from the [enscripten toolchain](). This is a bunch of C++ code that
needs to be compiled, and to simplify the whole process, as well as create
reproduceable builds, we have created
[`cosmwasm-opt`](https://github.com/confio/cosmwasm-opt), which contains a
`Dockerfile` that you can use to run both `wasm-pack` and `wasm-opt`. To make
the build, just run the following in the project directory:

```sh
docker run --rm -u $(id -u):$(id -g) -v $(pwd):/code confio/cosmwasm-opt:0.4.1

du -h contract.wasm
```

Note that this always outputs the file as `contract.wasm`, not with the name of
the project (yes, every tool chain has a different output location). For the
hackatom sample, I now get down to 52kB, an 18% improvement. This is as far as
you can minimize the input, without removing actual functionality.

While we cannot trim down the wasm code anymore, we can still reduce the size a
bit for loading in blockchain transactions. We
[soon plan](https://github.com/confio/go-cosmwasm/issues/20) to allow gzip-ed
wasm in the transactions posted to the chain, which will further reduce gas cost
of the code upload. Check out this final output:

```sh
$ gzip -k contract.wasm
$ du -h contract.wasm*
52K     contract.wasm
20K     contract.wasm.gz
```

And there you have it. We have gone from 1.5MB for the naive build, to 72kB with
the standard minification tooling, all the way down to 20kB with very aggressive
trimming and compression. Less than 1.5% of the original size. This is indeed
something you can easily fit inside a transaction.
