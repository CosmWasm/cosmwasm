# Nested contracts test

This contract doesn't do anything. Actually will panic at runtime. It simply
asserts at compile-time that it is possible to use another contract as a
dependency without using hacks such as the `library` feature, and conditional
`#[entry_point]` compilation.
