use std::collections::BTreeMap;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Expr, Ident, Token,
};

pub fn generate_api_impl(input: KV) -> Expr {
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
pub struct KV(BTreeMap<Ident, Value>);

impl Parse for KV {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let pairs = input.parse_terminated::<Pair, Token![,]>(Pair::parse)?;
        Ok(Self(pairs.into_iter().map(|p| p.0).collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
