mod entry_point;
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

// function documented in cosmwasm-std
#[proc_macro_attribute]
pub fn entry_point(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    entry_point::entry_point_impl(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn hash_function(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attr.is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::from(attr),
            "Unexpected parameters",
        )
        .into_compile_error()
        .into();
    }

    hash_function::hash_function_impl(item.into()).into()
}
