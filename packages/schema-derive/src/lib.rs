//! Derive macros for cosmwasm-schema. For internal use only.
//!
//! CosmWasm is a smart contract platform for the Cosmos ecosystem.
//! For more information, see: <https://cosmwasm.cosmos.network>
mod cw_serde;
mod error;
mod generate_api;
mod query_responses;

use self::error::fallible_macro;
use quote::ToTokens;
use syn::parse_macro_input;

#[derive(Clone, Copy, Debug)]
enum SchemaBackend {
    CwSchema,
    JsonSchema,
}

impl syn::parse::Parse for SchemaBackend {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::LitStr = input.parse()?;
        match ident.value().as_str() {
            "cw_schema" => Ok(SchemaBackend::CwSchema),
            "json_schema" => Ok(SchemaBackend::JsonSchema),
            _ => Err(syn::Error::new(ident.span(), "Unknown schema backend")),
        }
    }
}

fallible_macro! {
    #[proc_macro_derive(QueryResponses, attributes(returns, query_responses))]
    pub fn query_responses_derive(
        input: proc_macro::TokenStream,
    ) -> syn::Result<proc_macro::TokenStream> {
        let input = syn::parse(input)?;
        let expanded = query_responses::query_responses_derive_impl(input)?;

        Ok(expanded.into_token_stream().into())
    }
}

#[proc_macro]
pub fn write_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as generate_api::Options);
    let expanded = generate_api::write_api_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro]
pub fn generate_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as generate_api::Options);
    let expanded =
        generate_api::generate_api_impl(input.schema_backend(), &input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

fallible_macro! {
    #[proc_macro_attribute]
    pub fn cw_serde(
        attr: proc_macro::TokenStream,
        input: proc_macro::TokenStream,
    ) -> syn::Result<proc_macro::TokenStream> {
        let options = syn::parse(attr)?;
        let input = syn::parse(input)?;

        let expanded = cw_serde::cw_serde_impl(options, input)?;

        Ok(expanded.into_token_stream().into())
    }
}
