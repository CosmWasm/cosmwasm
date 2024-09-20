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

/// Submit the hash of the function to a global inventory
///
/// These hashes affect whether the Wasm cache is regenerated or not
#[proc_macro_attribute]
pub fn hash_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    hash_function::hash_function_impl(attr.into(), item.into()).into()
}
