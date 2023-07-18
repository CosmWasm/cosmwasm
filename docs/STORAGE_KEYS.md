# Storage keys

CosmWasm provides a generic key value store to contract developers via the
`Storage` trait. This is powerful but the nature of low level byte operations
makes it hard to use for high level storage types. In this document we discuss
the foundations of storage key composition all the way up to cw-storage-plus.

In a simple world, all you need is a `&[u8]` key which you can get e.g. using
`&17u64.to_be_bytes()`. This is an 8 bytes key with an encoded integer. But if
you have multiple data types in your contract, you want to prefix those keys in
order to avoid collisions. A simple concatenation is not sufficient because you
want to avoid collisions when part of the prefixes and part of the key overlap.
E.g. `b"keya" | b"x"` and `b"key" | b"ax"` (`|` denotes concatenation) must not
have the same binary representation.

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

With the emerging cw-storage-plus we see two additions to that approach:

1. Manually creating the namespace and concatenating it with `concat` makes no
   sense anymore. Instead `namespace` and `key` are always provided and a
   composed database key is created.
2. Using a multi component namespace becomes the norm.

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

1. Remove the single component `to_length_prefixed` implementation and fully
   commit to the multi-component version. This shifts focus from the recursive
   implementation to the compatible iterative implementation.
2. Rename "namespaces" to just "namespace" and let one namespace have multiple
   components.
3. Adopt the combined namespace + key encoder `namespaces_with_key` from
   cw-storage-plus.
4. Add a decomposition implementation

Given the importance of Length-prefixed keys for the entire CosmWasm ecosystem,
those implementations should be maintained in cosmwasm-std. The generic approach
allows building all sorts of storage solutions on top of it and it allows
indexers to parse storage keys for all of them.
