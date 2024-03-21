mod cw_serde;
mod generate_api;
mod query_responses;

use manyhow::manyhow;
use quote::ToTokens;
use syn::parse_macro_input;

#[manyhow]
#[proc_macro_derive(QueryResponses, attributes(returns, query_responses))]
pub fn query_responses_derive(
    input: proc_macro::TokenStream,
) -> syn::Result<proc_macro::TokenStream> {
    let input = syn::parse(input)?;
    let expanded = query_responses::query_responses_derive_impl(input)?;

    Ok(expanded.into_token_stream().into())
}

#[manyhow]
#[proc_macro]
pub fn write_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as generate_api::Options);
    let expanded = generate_api::write_api_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

#[manyhow]
#[proc_macro]
pub fn generate_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as generate_api::Options);
    let expanded = generate_api::generate_api_impl(&input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

#[manyhow]
#[proc_macro_attribute]
pub fn cw_serde(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> syn::Result<proc_macro::TokenStream> {
    let input = syn::parse(input)?;
    let expanded = cw_serde::cw_serde_impl(input)?;
    Ok(expanded.into_token_stream().into())
}
