#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

# List of contracts to be checked using nightly Rust channel.
contracts=(
  contracts/floaty/
)

for dir in "${contracts[@]}"; do
  (
    echo -e "\e[1;34mCHANGE WORKING DIRECTORY\e[0m: \e[1;32m$dir\e[0m"
    cd "$dir" || exit 1

    echo -e "\e[1;34mCHECK FORMATTING\e[0m"
    cargo fmt -- --check

    echo -e "\e[1;34mRUN UNIT TESTS\e[0m"
    cargo test --lib --locked

    echo  -e "\e[1;34mBUILD WASM\e[0m"
    RUSTFLAGS="-C target-feature=+nontrapping-fptoint" cargo build --release --lib --locked --target wasm32-unknown-unknown

    echo -e "\e[1;34mRUN LINTER\e[0m"
    cargo clippy --all-targets --tests -- -D warnings

    echo -e "\e[1;34mRUN INTEGRATION TESTS\e[0m"
    cargo test --test integration --locked

    echo -e "\e[1;34mGENERATE SCHEMA\e[0m"
    cargo run --bin schema --locked

    echo -e "\e[1;34mENSURE SCHEMA IS UP-TO-DATE\e[0m"
    git diff --quiet ./schema
  )
done
