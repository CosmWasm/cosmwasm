#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

(
  cd packages/std/src/math
  curl -sS -L https://github.com/CosmWasm/cosmwasm/raw/main/packages/std/src/math/uint64.rs >uint64.rs
  curl -sS -L https://github.com/CosmWasm/cosmwasm/raw/main/packages/std/src/math/uint128.rs >uint128.rs
  curl -sS -L https://github.com/CosmWasm/cosmwasm/raw/main/packages/std/src/math/uint256.rs >uint256.rs
  curl -sS -L https://github.com/CosmWasm/cosmwasm/raw/main/packages/std/src/math/uint512.rs >uint512.rs
)
