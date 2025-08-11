#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

contracts=(
  contracts/burner/
  contracts/crypto-verify/
  contracts/cyberpunk/
  contracts/empty/
  contracts/hackatom/
  contracts/ibc2/
  contracts/ibc-callbacks/
  contracts/ibc-reflect/
  contracts/ibc-reflect-send/
  contracts/nested-contracts/
  contracts/queue/
  contracts/reflect/
  contracts/replier/
  contracts/staking/
  contracts/virus/
)

for dir in "${contracts[@]}"; do
  (
    echo -e "\e[1;34mCHANGE WORKING DIRECTORY\e[0m: \e[1;32m$dir\e[0m"
    cd "$dir" || exit 1

    echo -e "\e[1;34mCHECK FORMATTING\e[0m"
    cargo +1.82.0 fmt -- --check

    echo -e "\e[1;34mRUN UNIT TESTS\e[0m"
    cargo +1.82.0 test --lib --locked

    echo -e "\e[1;34mBUILD WASM\e[0m"
    cargo +1.82.0 build --release --lib --locked --target wasm32-unknown-unknown

    echo -e "\e[1;34mRUN LINTER\e[0m"
    cargo +1.82.0 clippy --all-targets --tests -- -D warnings

    echo -e "\e[1;34mRUN INTEGRATION TESTS\e[0m"
    cargo +1.82.0 test --test integration --locked

    echo -e "\e[1;34mGENERATE SCHEMA\e[0m"
    cargo +1.82.0 run --bin schema --locked

    echo -e "\e[1;34mENSURE SCHEMA IS UP-TO-DATE\e[0m"
    git diff --quiet ./schema
  ) &
done

wait
