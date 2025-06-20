#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

cargo fmt
(cd packages/crypto && cargo test)
# A test inside the std package expects backtraces to be disabled
(cd packages/std && RUST_BACKTRACE=0 cargo test --features iterator,cosmwasm_1_2)
(cd packages/schema && cargo test)
(cd packages/schema-derive && cargo test)
(cd packages/vm && cargo test --features iterator,stargate)
