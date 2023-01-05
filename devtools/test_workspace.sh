#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

cargo fmt
(cd packages/crypto && cargo test)
(cd packages/std && cargo test --features iterator,cosmwasm_1_2)
(cd packages/storage && cargo test --features iterator)
(cd packages/schema && cargo test)
(cd packages/schema-derive && cargo test)
(cd packages/vm && cargo test --features iterator,stargate)
