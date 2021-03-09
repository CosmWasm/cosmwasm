#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

for contract_dir in contracts/*/; do
  (
    cd "$contract_dir"
    cargo fmt
    cargo check --tests
    cargo unit-test
    cargo clippy --tests -- -D warnings
    cargo schema
    cargo wasm
    cargo integration-test
  )
done
