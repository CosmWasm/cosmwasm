#!/bin/sh

set -e

export PATH=$PATH:/root/.cargo/bin
outdir=$(mktemp -d)

echo wasm-pack build --out-dir "${outdir}"
wasm-pack build --out-dir "${outdir}" || true

echo wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm

echo "done"