//! Derive macros for cosmwasm-vm. For internal use only. No stability guarantees.
//!
//! CosmWasm is a smart contract platform for the Cosmos ecosystem.
//! For more information, see: <https://cosmwasm.cosmos.network>

mod hash_function;

macro_rules! maybe {
    ($result:expr) => {{
        match { $result } {
            Ok(val) => val,
            Err(err) => return err.into_compile_error(),
        }
    }};
}
use maybe;

/// Hash the function
///
/// # Example
///
/// ```rust
/// # use cosmwasm_vm_derive::hash_function;
/// #[hash_function(const_name = "HASH")]
/// fn foo() {
///    println!("Hello, world!");
/// }
/// ```
#[proc_macro_attribute]
pub fn hash_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    hash_function::hash_function_impl(attr.into(), item.into()).into()
}
