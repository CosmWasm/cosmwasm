#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

cargo fmt
(cd packages/crypto && cargo check && cargo clippy --all-targets -- -D warnings)
(cd packages/derive && cargo check && cargo clippy --all-targets -- -D warnings)
(
  cd packages/std
  # default, min, all
  cargo check
  cargo check --no-default-features
  cargo check --features abort,iterator,staking,stargate,cosmwasm_1_2
  cargo wasm-debug
  cargo wasm-debug --features iterator,staking,stargate
  cargo clippy --all-targets --features iterator,staking,stargate -- -D warnings
)
(cd packages/storage && cargo build && cargo clippy --all-targets --features iterator -- -D warnings)
(cd packages/schema && cargo build && cargo clippy --all-targets -- -D warnings)
(cd packages/schema-derive && cargo build && cargo clippy --all-targets -- -D warnings)
(cd packages/vm && cargo build --features iterator,stargate && cargo clippy --all-targets --features iterator,stargate -- -D warnings)
(cd packages/check && cargo build && cargo clippy --all-targets -- -D warnings)
