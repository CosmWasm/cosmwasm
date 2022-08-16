#!/bin/bash
set -e

WASMS="$@"
[ -z "$WASMS" ] && echo "Usage: $0 <file>.wasm..." && exit 1

PROFILE="debug"

PROG=$(basename $0 .sh)
BASE_DIR=$(dirname $0)/..

BIN="$BASE_DIR/../../target/$PROFILE/examples/$PROG"

if [ "$PROFILE" = "release" ]; then
  cargo build --release --features iterator --example $PROG
else
  cargo build --features iterator --example $PROG
fi
echo "\`check_contract\` will be removed from the next version of \`cosmwasm-vm\` - please use \`cw-check-contract\` instead." >&2
echo "> cargo install cw-check-contract" >&2

for W in $@; do
  echo -n "Checking $(basename "$W")... "
  $BIN $W
done
