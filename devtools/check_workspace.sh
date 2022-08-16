#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

cargo fmt
(cd packages/crypto && cargo check && cargo clippy --all-targets -- -D warnings)
(cd packages/derive && cargo check && cargo clippy --all-targets -- -D warnings)
(cd packages/profiler && cargo check && cargo clippy --all-targets -- -D warnings)
(
  cd packages/std
  cargo check
  cargo check --features iterator,staking,stargate
  cargo wasm-debug
  cargo wasm-debug --features iterator,staking,stargate
  cargo clippy --all-targets --features iterator,staking,stargate -- -D warnings
  cargo schema
)
(cd packages/storage && cargo build && cargo clippy --all-targets --features iterator -- -D warnings)
(cd packages/schema && cargo build && cargo clippy --all-targets -- -D warnings)
(cd packages/schema-derive && cargo build && cargo clippy --all-targets -- -D warnings)
(cd packages/vm && cargo build --features iterator,stargate && cargo clippy --all-targets --features iterator,stargate -- -D warnings)
