#!/bin/sh

set -e

outdir=$(mktemp -d)
wasm-pack build --out-dir "${outdir}"
wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
