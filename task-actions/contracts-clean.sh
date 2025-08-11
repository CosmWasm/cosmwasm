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
    echo -e "\e[1;34mCONTRACT\e[0m: \e[1;32m$(basename "$dir")\e[0m"
    cd "$dir" || exit 1

    echo -e "\e[1;34mclean\e[0m"
    cargo clean
  )
done
