Compiling

`cargo wasm` does a basic compilation step, you need to run `wasm-gc <file>.wasm` to shrink it down afterwards.

`wasm-pack build` does the wasm compilation and trimming in one step. See if it is worth it

Deep optimization: use wasm-opt.
Produces around additional 25% savings on top of wasm-pack.

(Note `cargo wasm` fails if we use wasm_bindgen decorator... )

**TODO**

Prepare wasm-pack and wasm-opt pipeline for "productivication"