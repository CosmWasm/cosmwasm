# Defining Entry Points to Wasm

## Exports

`exports` are the functions that we export to the outside world. After
compilation these will be the only entry points that the application can call
into the web assembly contract. In general, the runtime will define a fixed set
of `exports` that must be implemented by the contract.

To make an export in rust code, you can add the following lines:

```rust
#[no_mangle]
pub extern "C" fn double(n: i32) -> i32 {
    n * 2
}
```

Note that you need the `#[no_mangle]` directive to keep the naming, and declare
it as `pub extern "C"` to create a proper C ABI, which is the standard interface
for Web Assembly, as well as FFI.

## Imports

If we want to interact with the outside world, the smart contract needs to
define a set of `imports`. These are function signatures we expect to be
implemented by the environment and provided when the VM is instantiated. If the
proper imports are not provided, you will receive an error upon instantiating
the contract.

```rust
extern "C" {
    fn c_read() -> *mut c_char;
    fn c_write(string: *mut c_char);
}
```

The above expects the runtime to provide read/write access to some (persistent)
singleton. Notably, the contract has no knowledge how it is stored, and no way
to "jailbreak" or access other element of the database.

## Memory Management

If you look closely, you will see every function definition in `exports` accepts
a fixed number of arguments of type `i32` and returns one result of type `i32`
(or `void`). With such limitations, how can one pass in a `struct` to the
contract, or even a serialized byte array (eg. json blob). And how can we return
a string back?

There is one more way in which the runtime can interact with a smart contract
instance. It can directly read and write to the linear memory of the smart
contract. In general, contracts are expected to export two well-defined
functions:

```rust
#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut c_void {
    let mut buffer = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    mem::forget(buffer);
    pointer as *mut c_void
}

#[no_mangle]
pub extern "C" fn deallocate(pointer: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, capacity);
    }
}
```

`allocate` heap allocates `size` bytes and tells the wasm alloc library not to
clear it (forget), after which it returns the pointer (integer offset) to the
caller. The caller can now safely write up to `size` bytes to the given offset,
eg. `copy(data, vm.Memory[offset:offset+size])`. Of course, this passes the
responsibility of freeing the memory from the wasm code to the caller, so make
sure to call `deallocate()` on the memory reference after the function call
finishes.

We can explore more complex and idiomatic ways of passing data between the
environment and the wasm contract, but for now, just ensure you export these two
functions and the runtime will make use of them to get `string` and `[]byte`
into and out of the wasm contract.
