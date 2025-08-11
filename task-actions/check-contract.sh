#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

message() {
  echo -e "\e[1;34m$1\e[0m \e[1;32m$2\e[0m"
}

check_contract_stable() {
  toolchain=1.82.0
  (
    contract_dir=$1
    contract="$(basename "$contract_dir" | tr - _)"

    message "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    message "CHECK FORMATTING" "$contract"
    cargo +$toolchain fmt -- --check

    message "RUN UNIT TESTS" "$contract"
    cargo +$toolchain test --lib --locked

    message "BUILD WASM" "$contract"
    cargo +$toolchain build --release --lib --locked --target wasm32-unknown-unknown

    message "RUN LINTER" "$contract"
    cargo +$toolchain clippy --all-targets --tests -- -D warnings

    message "RUN INTEGRATION TESTS" "$contract"
    cargo +$toolchain test --test integration --locked

    message "GENERATE SCHEMA" "$contract"
    cargo +$toolchain run --bin schema --locked

    message "ENSURE SCHEMA IS UP-TO-DATE" "$contract"
    git diff --quiet ./schema
  )
}

check_contract_nightly() {
  toolchain=nightly-2024-09-01
  (
    contract_dir=$1
    contract="$(basename "$contract_dir" | tr - _)"

    message "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    message "CHECK FORMATTING" "$contract"
    cargo +$toolchain fmt -- --check

    message "RUN UNIT TESTS" "$contract"
    cargo +$toolchain test --lib --locked

    message "BUILD WASM" "$contract"
    RUSTFLAGS="-C target-feature=+nontrapping-fptoint" cargo +$toolchain build --release --lib --locked --target wasm32-unknown-unknown

    message "RUN LINTER" "$contract"
    cargo +$toolchain clippy --all-targets --tests -- -D warnings

    message "RUN INTEGRATION TESTS" "$contract"
    cargo +$toolchain test --test integration --locked

    message "GENERATE SCHEMA" "$contract"
    cargo +$toolchain run --bin schema --locked

    message "ENSURE SCHEMA IS UP-TO-DATE" "$contract"
    git diff --quiet ./schema
  )
}
