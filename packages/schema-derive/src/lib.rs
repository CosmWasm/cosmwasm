use std::collections::BTreeMap;

use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Expr, ExprTuple, Ident, ItemEnum, ItemImpl, Token, Type,
    Variant,
};

/// Extract the query -> response mapping out of an enum variant.
fn parse_query(v: Variant) -> (String, Expr) {
    let query = to_snake_case(&v.ident.to_string());
    let response_ty: Type = v
        .attrs
        .iter()
        .find(|a| a.path.get_ident().unwrap() == "returns")
        .unwrap()
        .parse_args()
        .unwrap();

    (
        query,
        parse_quote!(cosmwasm_schema::schema_for!(#response_ty)),
    )
}

fn parse_tuple((q, r): (String, Expr)) -> ExprTuple {
    parse_quote! {
        (#q.to_string(), #r)
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
    let mappings = input.variants.into_iter().map(parse_query);
    let mut queries: Vec<_> = mappings.clone().map(|(q, _)| q).collect();
    queries.sort();
    let mappings = mappings.map(parse_tuple);

    parse_quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl cosmwasm_schema::QueryResponses for #ident {
            fn response_schemas_impl() -> std::collections::BTreeMap<String, schemars::schema::RootSchema> {
                std::collections::BTreeMap::from([
                    #( #mappings, )*
                ])
            }
        }
    }
}

#[proc_macro]
pub fn generate_api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as KV);

    let expanded = generate_api_impl(input).into_token_stream();

    proc_macro::TokenStream::from(expanded)
}

fn generate_api_impl(input: KV) -> Expr {
    let mut input = input.0;

    let name = if let Some(name_override) = input.remove(&parse_quote!(name)) {
        let name_override = name_override.unwrap_str();
        quote! {
            #name_override.to_string()
        }
    } else {
        quote! {
            env!("CARGO_PKG_NAME").to_string()
        }
    };

    let version = if let Some(version_override) = input.remove(&parse_quote!(version)) {
        let version_override = version_override.unwrap_str();
        quote! {
            #version_override.to_string()
        }
    } else {
        quote! {
            env!("CARGO_PKG_VERSION").to_string()
        }
    };

    let instantiate = input
        .remove(&parse_quote!(instantiate))
        .unwrap()
        .unwrap_type();

    let execute = match input.remove(&parse_quote!(execute)) {
        Some(ty) => {
            let ty = ty.unwrap_type();
            quote! {Some(schema_for!(#ty))}
        }
        None => quote! { None },
    };

    let (query, responses) = match input.remove(&parse_quote!(query)) {
        Some(ty) => {
            let ty = ty.unwrap_type();
            (
                quote! {Some(schema_for!(#ty))},
                quote! { Some(#ty::response_schemas().unwrap()) },
            )
        }
        None => (quote! { None }, quote! { None }),
    };

    let migrate = match input.remove(&parse_quote!(migrate)) {
        Some(ty) => {
            let ty = ty.unwrap_type();
            quote! {Some(schema_for!(#ty))}
        }
        None => quote! { None },
    };

    let sudo = match input.remove(&parse_quote!(sudo)) {
        Some(ty) => {
            let ty = ty.unwrap_type();
            quote! {Some(schema_for!(#ty))}
        }
        None => quote! { None },
    };

    parse_quote! {
        Api {
            contract_name: #name,
            contract_version: #version,
            instantiate: schema_for!(#instantiate),
            execute: #execute,
            query: #query,
            migrate: #migrate,
            sudo: #sudo,
            responses: #responses,
        }
    }
}

#[derive(Debug)]
enum Value {
    Type(syn::Path),
    Str(syn::LitStr),
}

impl Value {
    fn unwrap_type(self) -> syn::Path {
        if let Self::Type(p) = self {
            p
        } else {
            panic!("expected a type");
        }
    }

    fn unwrap_str(self) -> syn::LitStr {
        if let Self::Str(s) = self {
            s
        } else {
            panic!("expected a string literal");
        }
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        if let Ok(p) = input.parse::<syn::Path>() {
            Ok(Self::Type(p))
        } else {
            Ok(Self::Str(input.parse::<syn::LitStr>()?))
        }
    }
}

#[derive(Debug)]
struct Pair((Ident, Value));

impl Parse for Pair {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let k = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;
        let v = input.parse::<Value>()?;

        Ok(Self((k, v)))
    }
}

#[derive(Debug)]
struct KV(BTreeMap<Ident, Value>);

impl Parse for KV {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let pairs = input.parse_terminated::<Pair, Token![,]>(Pair::parse)?;
        Ok(Self(pairs.into_iter().map(|p| p.0).collect()))
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
                #[returns(some_crate::AnotherType)]
                Supply {},
                #[returns(SomeType)]
                Balance {},
            }
        };

        assert_eq!(
            query_responses_derive_impl(input),
            parse_quote! {
                #[automatically_derived]
                #[cfg(not(target_arch = "wasm32"))]
                impl cosmwasm_schema::QueryResponses for QueryMsg {
                    fn response_schemas_impl() -> std::collections::BTreeMap<String, schemars::schema::RootSchema> {
                        std::collections::BTreeMap::from([
                            ("supply".to_string(), cosmwasm_schema::schema_for!(some_crate::AnotherType)),
                            ("balance".to_string(), cosmwasm_schema::schema_for!(SomeType)),
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
            parse_tuple(parse_query(variant)),
            parse_quote! {
                ("get_foo".to_string(), cosmwasm_schema::schema_for!(Foo))
            }
        );

        let variant = parse_quote! {
            #[returns(some_crate::Foo)]
            GetFoo {}
        };

        assert_eq!(
            parse_tuple(parse_query(variant)),
            parse_quote! { ("get_foo".to_string(), cosmwasm_schema::schema_for!(some_crate::Foo)) }
        );
    }

    #[test]
    fn to_snake_case_works() {
        assert_eq!(to_snake_case("SnakeCase"), "snake_case");
        assert_eq!(to_snake_case("Wasm123AndCo"), "wasm123_and_co");
    }

    #[test]
    fn generate_api_minimal() {
        assert_eq!(
            generate_api_impl(parse_quote! {
                instantiate: InstantiateMsg,
            }),
            parse_quote! {
                Api {
                    contract_name: env!("CARGO_PKG_NAME").to_string(),
                    contract_version: env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: schema_for!(InstantiateMsg),
                    execute: None,
                    query: None,
                    migrate: None,
                    sudo: None,
                    responses: None,
                }
            }
        );
    }

    #[test]
    fn generate_api_name_vesion_override() {
        assert_eq!(
            generate_api_impl(parse_quote! {
                name: "foo",
                version: "bar",
                instantiate: InstantiateMsg,
            }),
            parse_quote! {
                Api {
                    contract_name: "foo".to_string(),
                    contract_version: "bar".to_string(),
                    instantiate: schema_for!(InstantiateMsg),
                    execute: None,
                    query: None,
                    migrate: None,
                    sudo: None,
                    responses: None,
                }
            }
        );
    }

    #[test]
    fn generate_api_all_msgs() {
        assert_eq!(
            generate_api_impl(parse_quote! {
                instantiate: InstantiateMsg,
                execute: ExecuteMsg,
                query: QueryMsg,
                migrate: MigrateMsg,
                sudo: SudoMsg,
            }),
            parse_quote! {
                Api {
                    contract_name: env!("CARGO_PKG_NAME").to_string(),
                    contract_version: env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: schema_for!(InstantiateMsg),
                    execute: Some(schema_for!(ExecuteMsg)),
                    query: Some(schema_for!(QueryMsg)),
                    migrate: Some(schema_for!(MigrateMsg)),
                    sudo: Some(schema_for!(SudoMsg)),
                    responses: Some(QueryMsg::response_schemas().unwrap()),
                }
            }
        );
    }
}
