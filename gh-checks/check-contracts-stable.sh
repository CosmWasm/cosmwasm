#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

for dir in contracts/*/; do
(
  cd "$dir"
  pwd
)
done
