#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

for contract_dir in contracts/*/; do
  (
    echo "Updating schema for $contract_dir"
    cd "$contract_dir"
    rm -r ./schema || true # ensure outdated schema files are deleted
    cargo schema
  )
done
