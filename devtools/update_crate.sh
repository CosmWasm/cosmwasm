#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

CRATE_NAME="$1"

# Update root Cargo.lock
cargo update -p "$CRATE_NAME"

for contract_dir in contracts/*/; do
  (
    cd "$contract_dir"
    cargo update -p "$CRATE_NAME"
  )
done
