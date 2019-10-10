#!/bin/sh

set -e

export PATH=$PATH:/root/.cargo/bin
echo $PATH
which wasm-pack
which wasm-opt

outdir=$(mktemp -d)
echo
pwd
ls
echo wasm-pack build --out-dir "${outdir}"

wasm-pack build --out-dir "${outdir}" || true
wasm-pack build --out-dir "${outdir}" || true
echo wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
wasm-opt -Os "${outdir}"/*.wasm -o contract.wasm
echo "done"