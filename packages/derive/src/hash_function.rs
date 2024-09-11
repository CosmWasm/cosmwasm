use proc_macro2::TokenStream;

pub fn hash_function_impl(input: TokenStream) -> TokenStream {
    let display = input.to_string();
    let hex_hash = blake3::hash(display.as_bytes()).to_hex();

    input
}