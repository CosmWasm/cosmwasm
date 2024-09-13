use proc_macro2::TokenStream;
use quote::quote;

use super::{bail, maybe};

pub fn hash_function_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        bail!(attr, "Unexpected parameters");
    }

    // Just verify that this is actually a function
    let _: syn::ItemFn = maybe!(syn::parse2(input.clone()));

    let display = input.to_string();
    let hex_hash = blake3::hash(display.as_bytes()).to_hex();
    let hex_hash = hex_hash.as_str();

    quote! {
        #input

        ::cosmwasm_vm_derive::inventory::submit! {
            ::cosmwasm_vm_derive::Hash(#hex_hash)
        }
    }
}
