#!/usr/bin/env bash

# Move to the workspace root
WORKSPACE_PATH=$(dirname $(cargo locate-project --workspace --message-format=plain))
cd $WORKSPACE_PATH

cargo build

for contract_dir in contracts/*/; do
  (cd "$contract_dir" && cargo build)
done
