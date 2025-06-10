use std::collections::BTreeMap;

use super::SchemaBackend;
use crate::error::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Block, ExprStruct, Ident, Token,
};

fn generate_api_write(api_object: syn::ExprStruct, name: &TokenStream) -> TokenStream {
    quote! {{
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
    }}
}

pub fn write_api_impl(input: Options) -> Block {
    let cw_api_object = generate_api_impl(SchemaBackend::CwSchema, &input);
    let json_schema_api_object = generate_api_impl(SchemaBackend::JsonSchema, &input);

    let crate_name = input.crate_name;
    let name = input.name;

    let cw_api_write = generate_api_write(cw_api_object, &name);
    let json_schema_api_write = generate_api_write(json_schema_api_object, &name);

    parse_quote! {
        {
            #[cfg(target_arch = "wasm32")]
            compile_error!("can't compile schema generator for the `wasm32` arch\nhint: are you trying to compile a smart contract without specifying `--lib`?");
            use ::std::env;
            use ::std::fs::{create_dir_all, write};

            use #crate_name::{remove_schemas, CwApi, Api, QueryResponses};

            let mut out_dir = env::current_dir().unwrap();
            out_dir.push("schema");
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            #json_schema_api_write

            out_dir.push("cw_schema");
            create_dir_all(&out_dir).unwrap();
            remove_schemas(&out_dir).unwrap();

            #cw_api_write
        }
    }
}

pub fn generate_api_impl(backend: SchemaBackend, input: &Options) -> ExprStruct {
    let Options {
        crate_name,
        name,
        version,
        ..
    } = input;

    let instantiate = input.instantiate(backend);
    let execute = input.execute(backend);
    let query = input.query(backend);
    let migrate = input.migrate(backend);
    let sudo = input.sudo(backend);
    let responses = input.responses(backend);

    let api_path = match backend {
        SchemaBackend::CwSchema => quote! { #crate_name::CwApi },
        SchemaBackend::JsonSchema => quote! { #crate_name::Api },
    };

    parse_quote! {
        #api_path {
            contract_name: #name.to_string(),
            contract_version: #version.to_string(),
            instantiate: #instantiate,
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
    fn get_type(self) -> syn::Result<syn::Path> {
        match self {
            Self::Type(p) => Ok(p),
            Self::Str(other) => bail!(other, "expected a type"),
        }
    }

    fn get_str(self) -> syn::Result<syn::LitStr> {
        match self {
            Self::Str(p) => Ok(p),
            Self::Type(other) => bail!(other, "expected a string literal"),
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

macro_rules! option_dispatch {
    ($opt:expr, $closure:expr) => {{
        match $opt {
            Some(ref ty) => {
                #[allow(clippy::redundant_closure_call)]
                let tokens = $closure(ty);
                quote! { Some(#tokens) }
            }
            None => quote! { None },
        }
    }};
}

macro_rules! backend_dispatch {
    ($fn_name:ident, $ty_field:ident) => {
        pub fn $fn_name(&self, backend: SchemaBackend) -> TokenStream {
            let crate_name = &self.crate_name;

            option_dispatch!(self.$ty_field, |ty| {
                match backend {
                    SchemaBackend::CwSchema => {
                        quote! { #crate_name::cw_schema::schema_of::<#ty>() }
                    }
                    SchemaBackend::JsonSchema => quote! { #crate_name::schema_for!(#ty) },
                }
            })
        }
    };
}

#[derive(Debug)]
pub struct Options {
    crate_name: syn::Path,
    name: TokenStream,
    version: TokenStream,
    instantiate_ty: Option<syn::Path>,
    execute_ty: Option<syn::Path>,
    query_ty: Option<syn::Path>,
    migrate_ty: Option<syn::Path>,
    sudo_ty: Option<syn::Path>,

    schema_backend: SchemaBackend,
}

impl Options {
    backend_dispatch!(instantiate, instantiate_ty);
    backend_dispatch!(execute, execute_ty);
    backend_dispatch!(query, query_ty);
    backend_dispatch!(migrate, migrate_ty);
    backend_dispatch!(sudo, sudo_ty);

    pub fn responses(&self, backend: SchemaBackend) -> TokenStream {
        let crate_name = &self.crate_name;

        option_dispatch!(self.query_ty, |ty| {
            match backend {
                SchemaBackend::CwSchema => {
                    quote! { <#ty as #crate_name::QueryResponses>::response_schemas_cw().unwrap() }
                }
                SchemaBackend::JsonSchema => {
                    quote! { <#ty as #crate_name::QueryResponses>::response_schemas().unwrap() }
                }
            }
        })
    }

    pub fn schema_backend(&self) -> SchemaBackend {
        self.schema_backend
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let pairs = input.parse_terminated(Pair::parse, Token![,])?;
        let mut map: BTreeMap<_, _> = pairs.into_iter().map(|p| p.0).collect();

        let crate_name = if let Some(crate_name_override) = map.remove(&parse_quote!(crate_name)) {
            crate_name_override.get_type()?
        } else {
            parse_quote! { ::cosmwasm_schema }
        };

        let name = if let Some(name_override) = map.remove(&parse_quote!(name)) {
            let name_override = name_override.get_str()?;
            quote! {
                #name_override
            }
        } else {
            quote! {
                ::std::env!("CARGO_PKG_NAME")
            }
        };

        let version = if let Some(version_override) = map.remove(&parse_quote!(version)) {
            let version_override = version_override.get_str()?;
            quote! {
                #version_override
            }
        } else {
            quote! {
                ::std::env!("CARGO_PKG_VERSION")
            }
        };

        let instantiate_ty = map
            .remove(&parse_quote!(instantiate))
            .map(|ty| ty.get_type())
            .transpose()?;

        let execute_ty = map
            .remove(&parse_quote!(execute))
            .map(|ty| ty.get_type())
            .transpose()?;

        let query_ty = map
            .remove(&parse_quote!(query))
            .map(|ty| ty.get_type())
            .transpose()?;

        let migrate_ty = map
            .remove(&parse_quote!(migrate))
            .map(|ty| ty.get_type())
            .transpose()?;

        let sudo_ty = map
            .remove(&parse_quote!(sudo))
            .map(|ty| ty.get_type())
            .transpose()?;

        let schema_backend = if let Some(backend) = map.remove(&parse_quote!(schema_backend)) {
            let backend = backend.get_str()?;
            parse_quote! { #backend }
        } else {
            SchemaBackend::JsonSchema
        };

        if let Some((invalid_option, _)) = map.into_iter().next() {
            bail!(invalid_option, "unknown generate_api option");
        }

        Ok(Self {
            schema_backend,
            crate_name,
            name,
            version,
            instantiate_ty,
            execute_ty,
            query_ty,
            migrate_ty,
            sudo_ty,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_rename() {
        assert_eq!(
            generate_api_impl(
                SchemaBackend::JsonSchema,
                &parse_quote! {
                    crate_name: ::my_crate::cw_schema,
                    instantiate: InstantiateMsg,
                    execute: ExecuteMsg,
                    query: QueryMsg,
                    migrate: MigrateMsg,
                    sudo: SudoMsg,
                }
            ),
            parse_quote! {
                ::my_crate::cw_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: Some(::my_crate::cw_schema::schema_for!(InstantiateMsg)),
                    execute: Some(::my_crate::cw_schema::schema_for!(ExecuteMsg)),
                    query: Some(::my_crate::cw_schema::schema_for!(QueryMsg)),
                    migrate: Some(::my_crate::cw_schema::schema_for!(MigrateMsg)),
                    sudo: Some(::my_crate::cw_schema::schema_for!(SudoMsg)),
                    responses: Some(<QueryMsg as ::my_crate::cw_schema::QueryResponses>::response_schemas().unwrap()),
                }
            }
        );
    }

    #[test]
    fn api_object_minimal() {
        assert_eq!(
            generate_api_impl(SchemaBackend::JsonSchema, &parse_quote! {}),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: None,
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
    fn api_object_instantiate_only() {
        assert_eq!(
            generate_api_impl(
                SchemaBackend::JsonSchema,
                &parse_quote! {
                    instantiate: InstantiateMsg,
                }
            ),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: Some(::cosmwasm_schema::schema_for!(InstantiateMsg)),
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
    fn api_object_name_version_override() {
        assert_eq!(
            generate_api_impl(
                SchemaBackend::JsonSchema,
                &parse_quote! {
                    name: "foo",
                    version: "bar",
                    instantiate: InstantiateMsg,
                }
            ),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: "foo".to_string(),
                    contract_version: "bar".to_string(),
                    instantiate: Some(::cosmwasm_schema::schema_for!(InstantiateMsg)),
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
            generate_api_impl(
                SchemaBackend::JsonSchema,
                &parse_quote! {
                    instantiate: InstantiateMsg,
                    execute: ExecuteMsg,
                    query: QueryMsg,
                    migrate: MigrateMsg,
                    sudo: SudoMsg,
                }
            ),
            parse_quote! {
                ::cosmwasm_schema::Api {
                    contract_name: ::std::env!("CARGO_PKG_NAME").to_string(),
                    contract_version: ::std::env!("CARGO_PKG_VERSION").to_string(),
                    instantiate: Some(::cosmwasm_schema::schema_for!(InstantiateMsg)),
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
    #[should_panic(expected = "unknown generate_api option")]
    fn invalid_option() {
        let _options: Options = parse_quote! {
            instantiate: InstantiateMsg,
            asd: Asd,
        };
    }
}
