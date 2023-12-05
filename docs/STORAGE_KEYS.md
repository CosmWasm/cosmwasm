# Storage keys

CosmWasm introduces a versatile key-value store accessible to contract developers
through the `Storage` trait. This low-level byte operation-based system, while powerful, 
can be challenging for managing high-level storage types. This documentation explores 
the evolution of storage key composition in CosmWasm, leading up to the current 
implementation in cw-storage-plus.

# The Challenge of Key Composition

The fundamental requirement for storage keys in CosmWasm is a `&[u8]` key, which
can be derived from basic types like integers (e.g., `&17u64.to_be_bytes()`). 
However, when handling various data types within a contract, it's crucial to 
use prefixed keys to prevent data collisions. 
Simple concatenation of keys is insufficient due to potential overlap issues. 
For instance, `b"keya" | b"x"` and `b"key" | b"ax"` should not yield the same binary representation, where | denotes concatenation.

# Evolution of Key Namespacing

In the early days, multiple approaches of key namespacing were discussed and
were documented here: https://github.com/webmaster128/key-namespacing. The "0x00
separated ASCIIHEX" approach was never used but "Length-prefixed keys" is used.

To recap, Length-prefixed keys have the following layout:

```
len(namespace_1) | namespace_1
  | len(namespace_2) | namespace_2
  | len(namespace_3) | namespace_3
  | ...
  | len(namespace_m) | namespace_m
  | key
```

In this repo (package `cosmwasm-storage`), the following functions were
implemented:

```rust
pub fn to_length_prefixed(namespace: &[u8]) -> Vec<u8>

pub fn to_length_prefixed_nested(namespaces: &[&[u8]]) -> Vec<u8>

fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8>
```

# Transition to `cw-storage-plus`
With the introduction of cw-storage-plus, there were significant enhancements:

1. **Simplified Key Composition:** The manual creation of namespaces followed by concatenation using concat was replaced by a more integrated approach, where namespace and key are provided together to create a composed database key.
2. **Multi-component Namespaces:** Using multiple components in a namespace became commonplace.

This led to the following addition in cw-storage-plus:

```rust
/// This is equivalent concat(to_length_prefixed_nested(namespaces), key)
/// But more efficient when the intermediate namespaces often must be recalculated
pub(crate) fn namespaces_with_key(namespaces: &[&[u8]], key: &[u8]) -> Vec<u8> {
```

In contrast to `concat(to_length_prefixed_nested(namespaces), key)` this direct
implementation saves once vector allocation since the final length can be
pre-computed and reserved. Also it's shorter to use.

Also since `to_length_prefixed` returns the same result as
`to_length_prefixed_nested` when called with one namespace element, there is no
good reason to preserve the single component version.

## 2023 updates

With the deprecation if cosmwasm-storage and the adoption of the system in
cw-storage-plus, it is time to do a few changes to the Length-prefixed keys
standard, without breaking existing users.

1. **Removal of Single Component Implementation:** The `to_length_prefixed` single-component version will be deprecated in favor of the multi-component `to_length_prefixed_nested` approach.
2. **Terminology Adjustment:** The term "namespaces" will be simplified to "namespace", encompassing multiple components.
3. **Adoption of Combined Encoder:** The `namespaces_with_key` function from `cw-storage-plus` will be the standard for key encoding.
4. **Decomposition Feature:** Introduction of a feature to decompose keys for enhanced flexibility.

Given the importance of Length-prefixed keys for the entire CosmWasm ecosystem,
those implementations should be maintained in cosmwasm-std. The generic approach
allows building all sorts of storage solutions on top of it and it allows
indexers to parse storage keys for all of them.
