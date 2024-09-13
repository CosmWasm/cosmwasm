use std::hash::{Hash, Hasher};

use proc_macro2::TokenStream;
use quote::quote;

use super::{bail, maybe};

struct Blake3Hasher {
    hasher: blake3::Hasher,
}

impl Blake3Hasher {
    fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    fn consume(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

impl Hasher for Blake3Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    fn finish(&self) -> u64 {
        unimplemented!();
    }
}

pub fn hash_function_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        bail!(attr, "Unexpected parameters");
    }

    // Just verify that this is actually a function
    let function: syn::ItemFn = maybe!(syn::parse2(input.clone()));

    let mut hasher = Blake3Hasher::new();
    function.hash(&mut hasher);

    let hex_hash = hasher.consume();

    quote! {
        #input

        ::cosmwasm_vm_derive::inventory::submit! {
            ::cosmwasm_vm_derive::Hash(#hex_hash)
        }
    }
}
