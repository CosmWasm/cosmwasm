#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

# Temporary incomplete testing command for development
(cd packages/vm \
  && cargo check --tests \
  && cargo check --features iterator --tests \
  && cargo test --features iterator calls:: \
  && cargo test --features iterator checksum:: \
  && cargo test --features iterator context:: \
  && cargo test --features iterator conversion:: \
  && cargo test --features iterator compatibility \
  && cargo test --features iterator errors:: \
  && cargo test --features iterator features:: \
  && cargo test --features iterator ffi:: \
  && cargo test --features iterator limiting_tunables:: \
  && cargo test --features iterator memory:: \
  && cargo test --features iterator modules:: \
  && cargo test --features iterator imports:: \
  && cargo test --features iterator testing:: \
  && cargo test --features iterator instance:: \
  && cargo test --features iterator cache:: \
  && cargo clippy --features iterator -- -D warnings)

# Contracts
for contract_dir in contracts/*/; do
  (cd "$contract_dir" && cargo wasm && cargo integration-test) || break;
done
