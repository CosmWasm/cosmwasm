#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

source ./task-actions/check-contract.sh

contracts_stable=(
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

for dir in "${contracts_stable[@]}"; do
  check_contract_stable "$dir"
done

contracts_nightly=(
  contracts/floaty/
)

for dir in "${contracts_nightly[@]}"; do
  check_contract_nightly "$dir"
done
