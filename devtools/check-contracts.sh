#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

msg() {
  printf "\033[1;34m%s\033[0m \033[1;32m%s\e[0m\n" "$1" "$2"
}

check_contract() {
  (
    contract_dir=$1
    package="$(basename "$contract_dir")"
    contract="$(echo "$package" | tr - _)"
    wasm="./target/wasm32-unknown-unknown/release/$contract.wasm"

    msg "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    msg "CHECK FORMATTING +$2" "$contract"
    cargo +"$2" fmt -- --check

    msg "RUN UNIT TESTS +$2" "$contract"
    cargo +"$2" test --lib #--locked

    msg "BUILD WASM +$2" "$contract"
    RUSTFLAGS="$4" cargo +"$2" build --release --lib --locked --target wasm32-unknown-unknown

    msg "RUN LINTER +$2" "$contract"
    cargo +"$2" clippy --all-targets --tests --locked -- -D warnings

    msg "RUN INTEGRATION TESTS +$3" "$contract"
    RUSTFLAGS="-A warnings" cargo +"$3" test \
                                  -p integration-"$package" \
                                  --test integration \
                                  --manifest-path=integration/Cargo.toml \
                                  #--locked

    msg "GENERATE SCHEMA +$2" "$contract"
    cargo +"$2" run --bin schema --locked

    msg "ENSURE SCHEMA IS UP-TO-DATE" "$contract"
    git diff --quiet ./schema

    msg "COSMWASM CHECK" "$contract"
    cosmwasm-check "$wasm"
  )
}

contracts_stable=(
  contracts/burner
  contracts/crypto-verify
  contracts/cyberpunk
  contracts/empty
  contracts/hackatom
  contracts/ibc-callbacks
  contracts/ibc-reflect
  contracts/ibc-reflect-send
  contracts/nested-contracts
  contracts/queue
  contracts/reflect
  contracts/staking
  contracts/virus
)

contracts_nightly=(
  contracts/floaty
)

toolchain_stable=1.81.0              # last Rust toolchain without 'reference-types'
toolchain_stable_integration=1.95.0  # Rust toolchain for running integration tests
rustflags_stable=""                  # no additional Rust flags needed

for dir in "${contracts_stable[@]}"; do
  check_contract "$dir" "$toolchain_stable" "$toolchain_stable_integration" "$rustflags_stable"
done

toolchain_nightly=nightly-2024-07-21                       # last Rust nightly version for 1.81.0
toolchain_nightly_integration=nightly-2026-02-28           # last Rust nightly version for 1.95.0
rustflags_nightly="-C target-feature=+nontrapping-fptoint" # additional Rust flags

for dir in "${contracts_nightly[@]}"; do
  check_contract "$dir" "$toolchain_nightly" "$toolchain_nightly_integration" "$rustflags_nightly"
done
