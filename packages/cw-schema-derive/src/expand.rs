use std::borrow::Cow;

use crate::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DataEnum, DataStruct, DataUnion, DeriveInput, Lit};

struct SerdeContainerOptions {
    rename_all: Option<syn::LitStr>,
    untagged: bool,
}

impl SerdeContainerOptions {
    fn parse(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let mut options = SerdeContainerOptions {
            rename_all: None,
            untagged: false,
        };

        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    options.rename_all = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("untagged") {
                    options.untagged = true;
                } else {
                    // TODO: support other serde attributes
                    //
                    // For now we simply clear the buffer to avoid errors
                    let _ = meta
                        .value()
                        .map(|val| val.parse::<TokenStream>().unwrap())
                        .unwrap_or_else(|_| meta.input.cursor().token_stream());
                }

                Ok(())
            })?;
        }

        Ok(options)
    }
}

struct ContainerOptions {
    crate_path: syn::Path,
}

impl ContainerOptions {
    fn parse(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let mut options = ContainerOptions {
            crate_path: syn::parse_str("::cw_schema")?,
        };

        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("schemaifier"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    let stringified: syn::LitStr = meta.value()?.parse()?;
                    options.crate_path = stringified.parse()?;
                } else {
                    bail!(meta.path, "unknown attribute");
                }

                Ok(())
            })?;
        }

        Ok(options)
    }
}

#[inline]
fn normalize_option<T: quote::ToTokens>(value: Option<T>) -> TokenStream {
    match value {
        Some(value) => quote! { Some(#value.into()) },
        None => quote! { None },
    }
}

fn extract_documentation(attributes: &[syn::Attribute]) -> syn::Result<Option<String>> {
    let docs_iter = attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
        .map(|doc_attribute| {
            let name_value = doc_attribute.meta.require_name_value()?;

            let syn::Expr::Lit(syn::ExprLit {
                lit: Lit::Str(ref text),
                ..
            }) = name_value.value
            else {
                bail!(name_value, "expected string literal");
            };

            Ok(Cow::Owned(text.value().trim().to_string()))
        });

    let docs = itertools::intersperse(docs_iter, Ok(Cow::Borrowed("\n")))
        .collect::<syn::Result<String>>()?;

    if docs.is_empty() {
        return Ok(None);
    }

    Ok(Some(docs))
}

fn patch_type_params<'a>(
    options: &ContainerOptions,
    type_params: impl Iterator<Item = &'a mut syn::TypeParam>,
) {
    let crate_path = &options.crate_path;

    for param in type_params {
        param.bounds.push(syn::TypeParamBound::Verbatim(
            quote! { #crate_path::Schemaifier },
        ));
    }
}

pub struct ContainerMeta {
    name: syn::Ident,
    description: Option<String>,
    generics: syn::Generics,
    options: ContainerOptions,
    serde_options: SerdeContainerOptions,
}

fn expand_enum(mut meta: ContainerMeta, input: DataEnum) -> syn::Result<TokenStream> {
    let crate_path = &meta.options.crate_path;

    let mut cases = Vec::new();
    for variant in input.variants.iter() {
        let value = match variant.fields {
            syn::Fields::Named(ref fields) => {
                let items = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap();
                    let description = normalize_option(extract_documentation(&field.attrs)?);
                    let field_ty = &field.ty;

                    let expanded = quote! {
                        (
                            stringify!(#name).into(),
                            #crate_path::StructProperty {
                                description: #description,
                                value: <#field_ty as #crate_path::Schemaifier>::visit_schema(visitor),
                            }
                        )
                    };

                    Ok(expanded)
                }).collect::<syn::Result<Vec<_>>>()?;

                quote! {
                    #crate_path::EnumValue::Named {
                        properties: #crate_path::reexport::BTreeMap::from([
                            #( #items, )*
                        ])
                    }
                }
            }
            syn::Fields::Unnamed(ref fields) => {
                let types = fields.unnamed.iter().map(|field| &field.ty);

                quote! {
                    #crate_path::EnumValue::Tuple {
                        items: vec![
                            #( <#types as #crate_path::Schemaifier>::visit_schema(visitor), )*
                        ]
                    }
                }
            }
            syn::Fields::Unit => quote! { #crate_path::EnumValue::Unit },
        };

        let variant_name = &variant.ident;
        let description = normalize_option(extract_documentation(&variant.attrs)?);

        let expanded = quote! {
            #crate_path::EnumCase {
                description: #description,
                value: #value,
            }
        };

        cases.push(quote! {
            (
                stringify!(#variant_name).into(),
                #expanded,
            )
        });
    }

    let name = &meta.name;
    let description = normalize_option(meta.description.as_ref());
    let crate_path = &meta.options.crate_path;

    patch_type_params(&meta.options, meta.generics.type_params_mut());
    let (impl_generics, ty_generics, where_clause) = meta.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::Schemaifier for #name #ty_generics #where_clause {
            fn visit_schema(visitor: &mut #crate_path::SchemaVisitor) -> #crate_path::DefinitionReference {
                let node = #crate_path::Node {
                    name: std::any::type_name::<Self>().into(),
                    description: #description,
                    value: #crate_path::NodeType::Enum {
                        discriminator: None,
                        cases: #crate_path::reexport::BTreeMap::from([
                            #( #cases, )*
                        ]),
                    },
                };

                visitor.insert(Self::id(), node)
            }
        }
    })
}

fn expand_struct(mut meta: ContainerMeta, input: DataStruct) -> syn::Result<TokenStream> {
    let name = &meta.name;
    let description = normalize_option(meta.description.as_ref());
    let crate_path = &meta.options.crate_path;

    let node_ty = match input.fields {
        syn::Fields::Named(named) => {
            let items = named.named.iter().map(|field| {
                let name = field.ident.as_ref().unwrap();
                let description = normalize_option(extract_documentation(&field.attrs)?);
                let field_ty = &field.ty;

                let expanded = quote! {
                    (
                        stringify!(#name).into(),
                        #crate_path::StructProperty {
                            description: #description,
                            value: <#field_ty as #crate_path::Schemaifier>::visit_schema(visitor),
                        }
                    )
                };

                Ok(expanded)
            }).collect::<syn::Result<Vec<_>>>()?;

            quote! {
                #crate_path::StructType::Named {
                    properties: #crate_path::reexport::BTreeMap::from([
                        #( #items, )*
                    ])
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let type_names = fields.unnamed.iter().map(|field| &field.ty);

            quote! {
                #crate_path::StructType::Tuple {
                    items: vec![
                        #(
                            <#type_names as #crate_path::Schemaifier>::visit_schema(visitor),
                        )*
                    ],
                }
            }
        }
        syn::Fields::Unit => quote! { #crate_path::StructType::Unit },
    };

    let node = quote! {
        #crate_path::Node {
            name: std::any::type_name::<Self>().into(),
            description: #description,
            value: #crate_path::NodeType::Struct(#node_ty),
        }
    };

    patch_type_params(&meta.options, meta.generics.type_params_mut());
    let (impl_generics, ty_generics, where_clause) = meta.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_path::Schemaifier for #name #ty_generics #where_clause {
            fn visit_schema(visitor: &mut #crate_path::SchemaVisitor) -> #crate_path::DefinitionReference {
                let node = {
                    #node
                };

                visitor.insert(Self::id(), node)
            }
        }
    })
}

fn expand_union(_meta: ContainerMeta, input: DataUnion) -> syn::Result<TokenStream> {
    Err(syn::Error::new_spanned(
        input.union_token,
        "Unions are not supported (yet)",
    ))
}

pub fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let options = ContainerOptions::parse(&input.attrs)?;
    let serde_options = SerdeContainerOptions::parse(&input.attrs)?;

    let description = extract_documentation(&input.attrs)?;

    let meta = ContainerMeta {
        name: input.ident,
        description,
        generics: input.generics,
        options,
        serde_options,
    };

    match input.data {
        syn::Data::Enum(input) => expand_enum(meta, input),
        syn::Data::Struct(input) => expand_struct(meta, input),
        syn::Data::Union(input) => expand_union(meta, input),
    }
}
