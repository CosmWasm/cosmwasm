#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

npx prettier@2.0.2 --write --prose-wrap always "./**/*.md"
