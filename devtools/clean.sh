#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

(cd packages/std && cargo clean)
(cd packages/crypto && cargo clean)
(cd packages/storage && cargo clean)
(cd packages/schema && cargo clean)
(cd packages/vm && cargo clean)
