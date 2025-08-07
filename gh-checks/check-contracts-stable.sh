#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

# List of contracts to be checked using stable Rust channel.
# Currently all except contract named 'floaty'.
contracts=()
for dir in contracts/*/; do
  [[ "$dir" == "contracts/floaty/" ]] && continue
  contracts+=("$dir")
done

for dir in "${contracts[@]}"; do
  (
    echo -e "\e[1;34mCHANGING WORKING DIRECTORY TO\e[0m: \e[1;32m$dir\e[0m"
    cd "$dir" || exit 1

    echo -e "\e[1;34mCHECKING FORMATTING\e[0m"
    cargo fmt -- --check

    echo -e "\e[1;34mRUNNING UNIT TESTS\e[0m"
    cargo test --lib --locked

    echo  -e "\e[1;34mBUILDING WASM BINARY\e[0m"
    cargo build --release --lib --locked --target wasm32-unknown-unknown

    echo -e "\e[1;34mRUNNING LINTER\e[0m"
    cargo clippy --all-targets --tests -- -D warnings

    echo -e "\e[1;34mRUNNING INTEGRATION TESTS\e[0m"
    cargo test --test integration --locked

    echo -e "\e[1;34mRUNNING SCHEMA GENERATOR\e[0m"
    cargo run --bin schema --locked

    echo -e "\e[1;34mENSURING SCHEMAS ARE UP-TO-DATE\e[0m"
    git diff --quiet ./schema
  )
done
