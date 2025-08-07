#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

for dir in contracts/*/; do
(
  echo "Change working directory to: $dir"
  cd "$dir"

  echo "Check formatting"
  cargo fmt -- --check

)
done
