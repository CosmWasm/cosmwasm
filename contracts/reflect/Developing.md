# Developing

If you have recently created a contract with this template, you probably could use some
help on how to build and test the contract, as well as prepare it for production. This
file attempts to provide a brief overview, assuming you have installed a recent
version of Rust already (eg. 1.40+).

## Prerequisites

Before starting, make sure you have [rustup](https://rustup.rs/) along with a
recent `rustc` and `cargo` version installed. Currently, we are testing on 1.40+.

And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Compiling and running tests

Now that you created your custom contract, make sure you can compile and run it before
making any changes. Go into the

```sh
# this will produce a wasm build in ./target/wasm32-unknown-unknown/release/YOUR_NAME_HERE.wasm
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACE=1 cargo unit-test

# this runs integration tests with cranelift backend (uses rust stable)
cargo integration-test

# this runs integration tests with singlepass backend (needs rust nightly)
cargo integration-test --no-default-features --features singlepass

# auto-generate json schema
cargo schema
```

The wasmer engine, embedded in `cosmwasm-vm` supports multiple backends:
singlepass and cranelift. Singlepass has fast compile times and slower run times,
and supportes gas metering. It also requires rust `nightly`. This is used as default
when embedding `cosmwasm-vm` in `go-cosmwasm` and is needed to use if you want to
check the gas usage.

However, when just building contacts, if you don't want to worry about installing
two rust toolchains, you can run all tests with cranelift. The integration tests
may take a small bit longer, but the results will be the same. The only difference
is that you can not check gas usage here, so if you wish to optimize gas, you must
switch to nightly and run with cranelift.

### Understanding the tests

The main code is in `src/contract.rs` and the unit tests there run in pure rust,
which makes them very quick to execute and give nice output on failures, especially
if you do `RUST_BACKTRACE=1 cargo unit-test`.

However, we don't just want to test the logic rust, but also the compiled Wasm artifact
inside a VM. You can look in `tests/integration.rs` to see some examples there. They
load the Wasm binary into the vm and call the contract externally. Effort has been
made that the syntax is very similar to the calls in the native rust contract and
quite easy to code. In fact, usually you can just copy a few unit tests and modify
a few lines to make an integration test (this should get even easier in a future release).

To run the latest integration tests, you need to explicitely rebuild the Wasm file with
`cargo wasm` and then run `cargo integration-test`.

We consider testing critical for anything on a blockchain, and recommend to always keep
the tests up to date. While doing active development, it is often simplest to disable
the integration tests completely and iterate rapidly on the code in `contract.rs`,
both the logic and the tests. Once the code is finalized, you can copy over some unit
tests into the integration.rs and make the needed changes. This ensures the compiled
Wasm also behaves as desired in the real system.

## Generating JSON Schema

While the Wasm calls (`init`, `handle`, `query`) accept JSON, this is not enough
information to use it. We need to expose the schema for the expected messages to the
clients. You can generate this schema by calling `cargo schema`, which will output
4 files in `./schema`, corresponding to the 3 message types the contract accepts,
as well as the internal `State`.

These files are in standard json-schema format, which should be usable by various
client side tools, either to auto-generate codecs, or just to validate incoming
json wrt. the defined schema.

## Preparing the Wasm bytecode for production

Before we upload it to a chain, we need to ensure the smallest output size possible,
as this will be included in the body of a transaction. We also want to have a
reproducible build process, so third parties can verify that the uploaded Wasm
code did indeed come from the claimed rust code.

To solve both these issues, we have produced `cosmwasm-opt`, a docker image to
produce an extremely small build output in a consistent manner. The suggest way
to run it is this:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source=$(basename "$(pwd)")_cache,target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  confio/cosmwasm-opt:0.7.3
```

We must mount the contract code to `/code`. You can use a absolute path instead
of `$(pwd)` if you don't want to `cd` to the directory first. The other two
volumes are nice for speedup. Mounting `/code/target` in particular is useful
to avoid docker overwriting your local dev files with root permissions.
Note the `/code/target` cache is unique for each contract being compiled to limit
interference, while the registry cache is global.

This is rather slow compared to local compilations, especially the first compile
of a given contract. The use of the two volume caches is very useful to speed up
following compiles of the same contract.

This produces a `contract.wasm` file in the current directory (which must be the root
directory of your rust project, the one with `Cargo.toml` inside). As well as
`hash.txt` containing the Sha256 hash of `contract.wasm`, and it will rebuild
your schema files as well.

### Testing production build

Once we have this compressed `contract.wasm`, we may want to ensure it is actually
doing everything it is supposed to (as it is about 4% of the original size).
If you update the "WASM" line in `tests/integration.rs`, it will run the integration
steps on the optimized build, not just the normal build. I have never seen a different
behavior, but it is nice to verify sometimes.

```rust
static WASM: &[u8] = include_bytes!("../contract.wasm");
```

Note that this is the same (deterministic) code you will be uploading to
a blockchain to test it out, as we need to shrink the size and produce a
clear mapping from wasm hash back to the source code.
