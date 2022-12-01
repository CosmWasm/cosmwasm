use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Block, ExprStruct, Ident, Path, Token,
};

pub fn write_api_impl(input: Options) -> Block {
    let api_object = generate_api_impl(&input);
    let name = input.name;

    parse_quote! {
        {
            #[cfg(target_arch = "wasm32")]
            compile_error!("can't compile schema generator for the `wasm32` arch\nhint: are you trying to compile a smart contract without specifying `--lib`?");
            use ::std::env;
            use ::std::fs::{create_dir_all, write};

            use ::cosmwasm_schema::{remove_schemas, Api, QueryResponses};

            let mut out_dir = env::current_dir().unwrap();
            out_dir.push("schema");
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            let api = #api_object.render();


            let path = out_dir.join(concat!(#name, ".json"));

            let json = api.to_string().unwrap();
            write(&path, json + "\n").unwrap();
            println!("Exported the full API as {}", path.to_str().unwrap());

            let raw_dir = out_dir.join("raw");
            create_dir_all(&raw_dir).unwrap();

            for (filename, json) in api.to_schema_files().unwrap() {
                let path = raw_dir.join(filename);

                write(&path, json + "\n").unwrap();
                println!("Exported {}", path.to_str().unwrap());
            }
        }
    }
}

pub fn generate_api_impl(input: &Options) -> ExprStruct {
    let Options {
        name,
        version,
        instantiate,
        execute,
        query,
        migrate,
        sudo,
        responses,
    } = input;

    parse_quote! {
        ::cosmwasm_schema::Api {
            contract_name: #name.to_string(),
            contract_version: #version.to_string(),
            instantiate: ::cosmwasm_schema::schema_for!(#instantiate),
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
pub struct Options {
    name: TokenStream,
    version: TokenStream,
    instantiate: Path,
    execute: TokenStream,
    query: TokenStream,
    migrate: TokenStream,
    sudo: TokenStream,
    responses: TokenStream,
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let pairs = input.parse_terminated::<Pair, Token![,]>(Pair::parse)?;
        let mut map: BTreeMap<_, _> = pairs.into_iter().map(|p| p.0).collect();

        let name = if let Some(name_override) = map.remove(&parse_quote!(name)) {
            let name_override = name_override.unwrap_str();
            quote! {
                #name_override
            }
        } else {
            quote! {
                ::std::env!("CARGO_PKG_NAME")
            }
        };

        let version = if let Some(version_override) = map.remove(&parse_quote!(version)) {
            let version_override = version_override.unwrap_str();
            quote! {
                #version_override
            }
        } else {
            quote! {
                ::std::env!("CARGO_PKG_VERSION")
            }
        };

        let instantiate = map
            .remove(&parse_quote!(instantiate))
            .unwrap()
            .unwrap_type();

        let execute = match map.remove(&parse_quote!(execute)) {
            Some(ty) => {
                let ty = ty.unwrap_type();
                quote! {Some(::cosmwasm_schema::schema_for!(#ty))}
            }
            None => quote! { None },
        };

        let (query, responses) = match map.remove(&parse_quote!(query)) {
            Some(ty) => {
                let ty = ty.unwrap_type();
                (
                    quote! {Some(::cosmwasm_schema::schema_for!(#ty))},
                    quote! { Some(<#ty as ::cosmwasm_schema::QueryResponses>::response_schemas().unwrap()) },
                )
            }
            None => (quote! { None }, quote! { None }),
        };

        let migrate = match map.remove(&parse_quote!(migrate)) {
            Some(ty) => {
                let ty = ty.unwrap_type();
                quote! {Some(::cosmwasm_schema::schema_for!(#ty))}
            }
            None => quote! { None },
        };

        let sudo = match map.remove(&parse_quote!(sudo)) {
            Some(ty) => {
                let ty = ty.unwrap_type();
                quote! {Some(::cosmwasm_schema::schema_for!(#ty))}
            }
            None => quote! { None },
        };

        if let Some((invalid_option, _)) = map.into_iter().next() {
            panic!("unknown generate_api option: {}", invalid_option);
        }

        Ok(Self {
            name,
            version,
            instantiate,
            execute,
            query,
            migrate,
            sudo,
            responses,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_object_minimal() {
        assert_eq!(
            generate_api_impl(&parse_quote! {
                instantiate: InstantiateMsg,
            }),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: ::cosmwasm_schema::schema_for!(InstantiateMsg),
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
    fn api_object_name_vesion_override() {
        assert_eq!(
            generate_api_impl(&parse_quote! {
                name: "foo",
                version: "bar",
                instantiate: InstantiateMsg,
            }),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: "foo".to_string(),
                    contract_version: "bar".to_string(),
                    instantiate: ::cosmwasm_schema::schema_for!(InstantiateMsg),
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
    fn api_object_all_msgs() {
        assert_eq!(
            generate_api_impl(&parse_quote! {
                instantiate: InstantiateMsg,
                execute: ExecuteMsg,
                query: QueryMsg,
                migrate: MigrateMsg,
                sudo: SudoMsg,
            }),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: ::cosmwasm_schema::schema_for!(InstantiateMsg),
                    execute: Some(::cosmwasm_schema::schema_for!(ExecuteMsg)),
                    query: Some(::cosmwasm_schema::schema_for!(QueryMsg)),
                    migrate: Some(::cosmwasm_schema::schema_for!(MigrateMsg)),
                    sudo: Some(::cosmwasm_schema::schema_for!(SudoMsg)),
                    responses: Some(<QueryMsg as ::cosmwasm_schema::QueryResponses>::response_schemas().unwrap()),
                }
            }
        );
    }

    #[test]
    #[should_panic(expected = "unknown generate_api option: asd")]
    fn invalid_option() {
        let _options: Options = parse_quote! {
            instantiate: InstantiateMsg,
            asd: Asd,
        };
    }
}
