Compiling

`cargo wasm` does a basic compilation step, you need to run `wasm-gc <file>.wasm` to shrink it down afterwards.

`wasm-pack build` does the wasm compilation and trimming in one step. See if it is worth it