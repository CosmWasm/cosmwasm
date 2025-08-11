#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

if [[ "${1:-}" == "parallel" ]]; then
  parallel=1
else
  parallel=0
fi

msg() {
  if (( !parallel )); then
    echo -e "\e[1;34m$1\e[0m \e[1;32m$2\e[0m"
  fi
}

check_contract_stable() {
  toolchain=1.82.0
  (
    contract_dir=$1
    contract="$(basename "$contract_dir" | tr - _)"
    wasm="./target/wasm32-unknown-unknown/release/$contract.wasm"

    msg "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    msg "CHECK FORMATTING" "$contract"
    cargo +$toolchain fmt -- --check

    msg "RUN UNIT TESTS" "$contract"
    cargo +$toolchain test --lib --locked

    msg "BUILD WASM" "$contract"
    cargo +$toolchain build --release --lib --locked --target wasm32-unknown-unknown

    msg "RUN LINTER" "$contract"
    cargo +$toolchain clippy --all-targets --tests -- -D warnings

    msg "RUN INTEGRATION TESTS" "$contract"
    cargo +$toolchain test --test integration --locked

    msg "GENERATE SCHEMA" "$contract"
    cargo +$toolchain run --bin schema --locked

    msg "ENSURE SCHEMA IS UP-TO-DATE" "$contract"
    git diff --quiet ./schema

    msg "cosmwasm-check (release)" "$contract"
    cosmwasm-check-release "$wasm"

    msg "cosmwasm-check (develop)" "$contract"
    cosmwasm-check "$wasm"
  )
}

check_contract_nightly() {
  toolchain=nightly-2024-09-01
  (
    contract_dir=$1
    contract="$(basename "$contract_dir" | tr - _)"
    wasm="./target/wasm32-unknown-unknown/release/$contract.wasm"

    msg "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    msg "CHECK FORMATTING" "$contract"
    cargo +$toolchain fmt -- --check

    msg "RUN UNIT TESTS" "$contract"
    cargo +$toolchain test --lib --locked

    msg "BUILD WASM" "$contract"
    RUSTFLAGS="-C target-feature=+nontrapping-fptoint" cargo +$toolchain build --release --lib --locked --target wasm32-unknown-unknown

    msg "RUN LINTER" "$contract"
    cargo +$toolchain clippy --all-targets --tests -- -D warnings

    msg "RUN INTEGRATION TESTS" "$contract"
    cargo +$toolchain test --test integration --locked

    msg "GENERATE SCHEMA" "$contract"
    cargo +$toolchain run --bin schema --locked

    msg "ENSURE SCHEMA IS UP-TO-DATE" "$contract"
    git diff --quiet ./schema

    msg "cosmwasm-check (release)" "$contract"
    cosmwasm-check-release "$wasm"

    msg "cosmwasm-check (develop)" "$contract"
    cosmwasm-check  "$wasm"
  )
}

contracts_stable=(
  contracts/burner
  contracts/crypto-verify
  contracts/cyberpunk
  contracts/empty
  contracts/hackatom
  contracts/ibc2
  contracts/ibc-callbacks
  contracts/ibc-reflect
  contracts/ibc-reflect-send
  contracts/nested-contracts
  contracts/queue
  contracts/reflect
  contracts/replier
  contracts/staking
  contracts/virus
)

contracts_nightly=(
  contracts/floaty
)

if (( parallel )); then
  for dir in "${contracts_stable[@]}"; do
    check_contract_stable "$dir" > /dev/null &
  done
  for dir in "${contracts_nightly[@]}"; do
    check_contract_nightly "$dir" > /dev/null &
  done
  wait
else
  for dir in "${contracts_stable[@]}"; do
    check_contract_stable "$dir"
  done
  for dir in "${contracts_nightly[@]}"; do
    check_contract_nightly "$dir"
  done
fi
