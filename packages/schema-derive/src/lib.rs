use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, ExprTuple, ItemEnum, ItemImpl, Type, Variant};

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(v: Variant) -> ExprTuple {
    let query = to_snake_case(&v.ident.to_string());
    let response_ty: Type = v
        .attrs
        .iter()
        .find(|a| a.path.get_ident().unwrap() == "returns")
        .unwrap()
        .parse_args()
        .unwrap();

    parse_quote! {
        (#query.to_string(), cosmwasm_schema::schema_for!(#response_ty))
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

    let expanded = query_responses_derive_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

fn query_responses_derive_impl(input: ItemEnum) -> ItemImpl {
    let ident = input.ident;

    let responses = input.variants.into_iter().map(parse_query);

    parse_quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl cosmwasm_schema::QueryResponses for #ident {
            fn response_schemas() -> std::collections::BTreeMap<String, schemars::schema::RootSchema> {
                std::collections::BTreeMap::from([
                    #( #responses, )*
                ])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn happy_path() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "snake_case")]
            pub enum QueryMsg {
                #[returns(SomeType)]
                Balance {},
                #[returns(some_crate::AnotherType)]
                Supply {},
            }
        };

        assert_eq!(
            query_responses_derive_impl(input),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl cosmwasm_schema::QueryResponses for QueryMsg {
                    fn response_schemas() -> std::collections::BTreeMap<String, schemars::schema::RootSchema> {
                        std::collections::BTreeMap::from([
                            ("balance".to_string(), cosmwasm_schema::schema_for!(SomeType)),
                            ("supply".to_string(), cosmwasm_schema::schema_for!(some_crate::AnotherType)),
                        ])
                    }
                }
            }
        );
    }

    #[test]
    fn parse_query_works() {
        let variant = parse_quote! {
            #[returns(Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_query(variant),
            parse_quote! {
                ("get_foo".to_string(), cosmwasm_schema::schema_for!(Foo))
            }
        );

        let variant = parse_quote! {
            #[returns(some_crate::Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_query(variant),
            parse_quote! { ("get_foo".to_string(), cosmwasm_schema::schema_for!(some_crate::Foo)) }
        );
    }

    #[test]
    fn to_snake_case_works() {
        assert_eq!(to_snake_case("SnakeCase"), "snake_case");
        assert_eq!(to_snake_case("Wasm123AndCo"), "wasm123_and_co");
    }

    #[test]
    #[should_panic]
    fn panic_if_queries_are_not_snake_case1() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            pub enum QueryMsg {
                #[returns(SomeType)]
                Balance {},
            }
        };

        query_responses_derive_impl(input);
    }

    #[test]
    #[should_panic]
    fn panic_if_queries_are_not_snake_case2() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(deny_unknown_fields)]
            pub enum QueryMsg {
                #[returns(SomeType)]
                Balance {},
            }
        };

        query_responses_derive_impl(input);
    }

    #[test]
    #[should_panic]
    fn panic_if_queries_are_not_snake_case3() {
        let input: ItemEnum = parse_quote! {
            #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
            #[serde(rename_all = "kebab-case")]
            pub enum QueryMsg {
                #[returns(SomeType)]
                Balance {},
            }
        };

        query_responses_derive_impl(input);
    }
}
