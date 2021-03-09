#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

for contract_dir in contracts/*/
do
  (cd "$contract_dir" && cargo clean)
done

(cd packages/std && cargo clean)
(cd packages/crypto &&  cargo clean)
(cd packages/storage &&  cargo clean)
(cd packages/schema &&  cargo clean)
(cd packages/vm &&  cargo clean)
