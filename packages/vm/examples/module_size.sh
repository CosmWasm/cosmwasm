#!/bin/bash
# Uses valgrind's massif tool to compute heap memory consumption of compiled modules.

WASM="$1"
[ -z "$WASM" ] && echo "Usage: $0 <file>.wasm" && exit 1

MEM_UTIL="valgrind --tool=massif --max-snapshots=10"
SUM_UTIL="ms_print"

PROG=`basename $0 .sh`
BASE_DIR=`dirname $0`/..

# Look for the useful info (compile_only() results)
FN="compile_only"

BIN="$BASE_DIR/../../target/release/examples/$PROG"

RESULTS="$BASE_DIR/$PROG.log"
SUMMARY="$BASE_DIR/ms_print.log"

RUSTFLAGS="-g" cargo build --release --example module_size

$MEM_UTIL --massif-out-file=$RESULTS $BIN $WASM
$SUM_UTIL $RESULTS >$SUMMARY

# Compute compile_only() total (heap) bytes
echo -n "module size (unserialized): "
tac $SUMMARY | sed -n '1,/^  n /p' | grep "::$FN " | cut -f2 -d\( | cut -f1 -d\) | sed 's/,//g;s/B//' | sed ':a;N;s/\n/+/;ta' | bc -l | sed 's/$/ bytes/'
