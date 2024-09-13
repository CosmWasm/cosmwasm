mod hash_function;
mod read_wasmer_version;

macro_rules! bail {
    ($span:ident, $message:literal) => {{
        return ::syn::Error::new_spanned($span, $message)
            .into_compile_error()
            .into();
    }};
    ($message:literal) => {{
        return ::syn::Error::new(proc_macro2::Span::call_site(), $message)
            .into_compile_error()
            .into();
    }};
}

macro_rules! maybe {
    ($result:expr) => {{
        match { $result } {
            Ok(val) => val,
            Err(err) => return err.into_compile_error(),
        }
    }};
}
use {bail, maybe};

#[proc_macro]
pub fn read_wasmer_version(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    read_wasmer_version::read_wasmer_version_impl(input.into()).into()
}

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