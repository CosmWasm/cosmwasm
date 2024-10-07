use std::hash::{Hash, Hasher};

use blake2::{Blake2b512, Digest};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Token};

use super::maybe;

struct Options {
    const_name: syn::Ident,
}

impl syn::parse::Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let params = Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut const_name = None;
        for param in params {
            if param.path.is_ident("const_name") {
                if const_name.is_some() {
                    return Err(syn::Error::new_spanned(param, "Duplicate parameter"));
                }

                let ident_as_string: syn::LitStr = syn::parse2(param.value.to_token_stream())?;
                const_name = Some(ident_as_string.parse()?);
            } else {
                return Err(syn::Error::new_spanned(param, "Unknown parameter"));
            }
        }

        Ok(Self {
            const_name: const_name
                .ok_or_else(|| syn::Error::new(input.span(), "Missing parameters"))?,
        })
    }
}

struct Blake2Hasher {
    hasher: Blake2b512,
}

impl Blake2Hasher {
    fn new() -> Self {
        Self {
            hasher: Blake2b512::new(),
        }
    }

    fn consume(self) -> [u8; 64] {
        self.hasher.finalize().into()
    }
}

impl Hasher for Blake2Hasher {
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    fn finish(&self) -> u64 {
        unimplemented!();
    }
}

pub fn hash_function_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let options: Options = maybe!(syn::parse2(attr));

    // Just verify that this is actually a function
    let function: syn::ItemFn = maybe!(syn::parse2(input.clone()));

    let mut hasher = Blake2Hasher::new();
    function.hash(&mut hasher);
    let hash = hasher.consume();

    let hash_variable_name = &options.const_name;
    let hash_bytes = hash.as_slice();

    quote! {
        pub const #hash_variable_name: &[u8] = &[#(#hash_bytes),*];

        #input
    }
}
