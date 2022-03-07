#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Running with -c makes the script only validate instead of editing in place.
op="w"
while getopts c option; do
  case "${option}" in
  c) op="d" ;;
  *) ;;
  esac
done

shfmt -$op devtools packages
