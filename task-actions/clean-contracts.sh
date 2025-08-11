#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

contracts=(
  contracts/burner/
  contracts/crypto-verify/
  contracts/cyberpunk/
  contracts/empty/
  contracts/floaty/
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
    contract="$(basename "$dir" | tr - _)"

    echo -e "\e[1;34mClean contract\e[0m \e[1;32m$contract\e[0m"
    cd "$dir" || exit 1

    cargo clean
  )
done
