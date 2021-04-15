#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

for contract_dir in contracts/*/; do
  (
    cd "$contract_dir"
    cargo fmt
    mkdir -p target/wasm32-unknown-unknown/release/
    touch target/wasm32-unknown-unknown/release/"$(basename "$contract_dir" | tr - _)".wasm
    cargo check --tests
    cargo unit-test
    touch src/*.rs # Needed because check and clippy share the same build cache such that clippy warnings are hidden when check wrote to the build cache
    cargo clippy --all-targets -- -D warnings
    cargo schema
    cargo wasm
    cargo integration-test
  )
done
