#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

msg() {
  printf "\033[1;34mClean contract\033[0m \033[1;32m%s\033[0m\n" "$1"
}

for dir in contracts/*/; do
  (
    contract="$(basename "$dir" | tr - _)"
    msg "$contract"
    cd "$dir" || exit 1
    cargo clean
  )
done
