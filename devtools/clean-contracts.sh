#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

for dir in contracts/*/; do
  (
    contract="$(basename "$dir" | tr - _)"
    echo -e "\e[1;34mClean contract\e[0m \e[1;32m$contract\e[0m"
    cd "$dir" || exit 1
    cargo clean
  )
done
