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
    echo -e "Changing working directory to \033[1;32m$dir\033[0m"
    cd "$dir" || exit 1

    echo "Checking formatting"
    cargo fmt -- --check

    echo "Running unit tests"
    cargo test --lib --locked --quiet

    echo  "Building WASM binary"
    cargo build --release --lib --locked --target wasm32-unknown-unknown

    echo "Running linter"
    cargo clippy --all-targets --tests -- -D warnings

    echo "Running integration tests"
    cargo test --test integration --locked --quiet

    echo "Running schema generator"
    cargo run --bin schema --locked

    echo "Ensuring schemas are up-to-date"
    git diff --quiet ./schema
  )
done
