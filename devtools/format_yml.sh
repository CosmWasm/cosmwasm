#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Running with -c makes the script only validate instead of editing in place.
op="write"
while getopts c option; do
  case "${option}" in
  c) op="check" ;;
  *) ;;
  esac
done

npx prettier@3.3.3 --$op "./**/*.yml"
