#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

rm Cargo.lock
cargo +nightly build -Zminimal-versions

for contract_dir in contracts/*/; do
  (
    cd "$contract_dir"
    rm Cargo.lock
    cargo +nightly build -Zminimal-versions
  )
done
