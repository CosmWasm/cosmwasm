#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

msg() {
  echo -e "\e[1;34m$1\e[0m \e[1;32m$2\e[0m"
}

check_contract() {
  (
    contract_dir=$1
    contract="$(basename "$contract_dir" | tr - _)"

    msg "CHANGE DIRECTORY" "$contract_dir"
    cd "$contract_dir" || exit 1

    msg "UPDATE SCHEMA" "$contract"
    cargo +"$2" run --bin schema --locked
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

toolchain_stable=1.82.0
toolchain_nightly=nightly-2024-09-01

for dir in "${contracts_stable[@]}"; do
  check_contract "$dir" "$toolchain_stable"
done
for dir in "${contracts_nightly[@]}"; do
  check_contract "$dir" "$toolchain_nightly"
done
