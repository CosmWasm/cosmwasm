mod generate_api;
mod query_responses;

use quote::ToTokens;
use syn::{parse_macro_input, ItemEnum};

#[proc_macro_derive(QueryResponses, attributes(returns))]
pub fn query_responses_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let expanded = query_responses::query_responses_derive_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as generate_api::KV);

    let expanded = generate_api::generate_api_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}
