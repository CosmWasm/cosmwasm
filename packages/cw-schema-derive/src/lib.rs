//! Derive macros for cw-schema. For internal use only.
//!
//! CosmWasm is a smart contract platform for the Cosmos ecosystem.
//! For more information, see: <https://cosmwasm.cosmos.network>

mod expand;

macro_rules! bail {
    ($span_src:expr, $msg:literal) => {{
        return Err($crate::error_message!($span_src, $msg));
    }};
}

macro_rules! error_message {
    ($span_src:expr, $msg:literal) => {{
        ::syn::Error::new(::syn::spanned::Spanned::span(&{ $span_src }), $msg)
    }};
}
// Needed so we can import macros. Rust, why?
use {bail, error_message};

#[proc_macro_derive(Schemaifier, attributes(schemaifier, serde))]
pub fn schemaifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match expand::expand(input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
