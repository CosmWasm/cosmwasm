# CHANGELOG

## HEAD

[Define canonical address callbacks](https://github.com/confio/cosmwasm/issues/73)

* Use `&[u8]` for addresses in params
* Allow contracts to resolve human readable addresses (`&str`) in their json into a fixed-size binary representation
* Provide mocks for unit testing and integration tests

## 0.5

### 0.5.2

This is the first documented and supported implementation. It contains the basic feature set.
`init` and `handle` supported for modules and can return messages. A stub implementation of 
`query` is done, which is likely to be deprecated soon. Some main points:

* The build-system and unit/integration-test setup is all stabilized.
* Cosmwasm-vm supports singlepass and cranelift backends, and caches modules on disk and instances in memory (lru cache).
* JSON Schema output works

All future Changelog entries will reference this base