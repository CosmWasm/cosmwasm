use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, Type, Variant};

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(v: Variant) -> TokenStream {
    let query = to_snake_case(&v.ident.to_string());
    let response_ty: Type = v
        .attrs
        .iter()
        .find(|a| a.path.get_ident().unwrap() == "returns")
        .unwrap()
        .parse_args()
        .unwrap();

    quote! {
        (#query, cosmwasm_schema::schema_for!(#response_ty))
    }
}

fn to_snake_case(input: &str) -> String {
    // this was stolen from serde for consistent behavior
    let mut snake = String::new();
    for (i, ch) in input.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

#[proc_macro_derive(QueryResponses, attributes(returns))]
pub fn query_responses_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let ident = input.ident;

    let responses = input.variants.into_iter().map(parse_query);

    let expanded = quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl cosmwasm_schema::QueryResponses for #ident {
            fn query_responses() -> std::collections::BTreeMap<&'static str, schemars::schema::RootSchema> {
                [
                    #( #responses, )*
                ]
                    .into_iter()
                    .collect()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[cfg(tests)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn parse_query() {
        let variant = parse_quote! {
            #[returns(Foo)]
            GetFoo {},
        };

        assert_eq!(
            parse_query(variant),
            quote! { ("get_foo", cosmwasm_schema::schema_for!(Foo)) }
        );

        let variant = parse_quote! {
            #[returns(some_crate::Foo)]
            GetFoo {},
        };

        assert_eq!(
            parse_query(variant),
            quote! { ("get_foo", cosmwasm_schema::schema_for!(some_crate::Foo)) }
        );
    }

    #[test]
    fn to_snake_case() {
        assert_eq!(to_snake_case("SnakeCase"), "snake_case");
        assert_eq!(to_snake_case("Wasm123AndCo"), "wasm123_and_co");
    }
}
