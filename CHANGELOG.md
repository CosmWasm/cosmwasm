# CHANGELOG

## HEAD

## 0.5

### 0.5.2

This is the first documented and supported implementation. It contains the basic feature set.
`init` and `handle` supported for modules and can return messages. A stub implementation of 
`query` is done, which is likely to be deprecated soon. Some main points:

* The build-system and unit/integration-test setup is all stabilized.
* Cosmwasm-vm supports singlepass and cranelift backends, and caches modules on disk and instances in memory (lru cache).
* JSON Schema output works

All future Changelog entries will reference this base