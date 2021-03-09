#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

cargo fmt
(cd packages/crypto && cargo build && cargo test && cargo clippy -- -D warnings)
(cd packages/std && cargo wasm-debug --features iterator && cargo test --features iterator && cargo clippy --features iterator -- -D warnings && cargo schema)
(cd packages/storage && cargo build && cargo test --features iterator && cargo clippy --features iterator -- -D warnings)
(cd packages/schema && cargo build && cargo test && cargo clippy -- -D warnings)
(cd packages/vm && cargo build --features iterator,stargate && cargo test --features iterator,stargate && cargo clippy --features iterator,stargate -- -D warnings)
