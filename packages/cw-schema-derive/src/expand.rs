// TODO: CLEAN ALL THIS SHIT UP WHAT THE FUCK IS THIS

use crate::bail;
use owo_colors::{OwoColorize, Style};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    borrow::Cow, env, fmt::Display, io::{self, Write as _}
};
use syn::{DataEnum, DataStruct, DataUnion, DeriveInput, Lit};

const DISABLE_WARNINGS_VAR: &str = "SHUT_UP_CW_SCHEMA_DERIVE";

fn print_warning(title: impl Display, content: impl Display) -> io::Result<()> {
    if let Ok("1") = env::var(DISABLE_WARNINGS_VAR).as_deref() {
        return Ok(());
    }

    let mut sink = io::stderr();

    let bold_yellow = Style::new().bold().yellow();
    let bold = Style::new().bold();
    let blue = Style::new().blue();

    write!(sink, "{}", "warning".style(bold_yellow))?;
    writeln!(
        sink,
        "{}",
        format_args!("({}): {title}", env!("CARGO_PKG_NAME")).style(bold)
    )?;

    writeln!(sink, "{}", "  | ".style(blue))?;
    write!(sink, "{}", "  | ".style(blue))?;
    writeln!(sink, "{content}")?;

    writeln!(sink, "{}", "  | ".style(blue))?;
    writeln!(sink, "{}", "  | ".style(blue))?;

    write!(sink, "{}", "  = ".style(blue))?;
    write!(sink, "{}", "note: ".style(bold))?;
    writeln!(sink, "set `{DISABLE_WARNINGS_VAR}=1` to silence this warning")?;

    Ok(())
}

type Converter = fn(&str) -> String;

fn case_converter(case: &syn::LitStr) -> syn::Result<Converter> {
    macro_rules! define_converter {
        (match $value:expr => {
            $( $case:pat => $converter:expr, )*
        }) => {
            match $value {
                $( $case => |txt: &str| $converter(txt).to_string(), )*
                _ => return Err(syn::Error::new_spanned(case, "unsupported case style")),
            }
        };
    }

    let case = case.value();
    let converter = define_converter!(match case.as_str() => {
        "camelCase" => heck::AsLowerCamelCase,
        "snake_case" => heck::AsSnakeCase,
        "kebab-case" => heck::AsKebabCase,
        "SCREAMING_SNAKE_CASE" => heck::AsShoutySnakeCase,
        "SCREAMING-KEBAB-CASE" => heck::AsShoutyKebabCase,
    });

    Ok(converter)
}

#[inline]
fn maybe_case_converter(case: Option<&syn::LitStr>) -> syn::Result<Converter> {
    case.map(case_converter)
        .unwrap_or_else(|| Ok(|txt: &str| txt.to_string()))
}

#[inline]
fn ident_adapter(converter: Converter) -> impl Fn(&syn::Ident) -> syn::Ident {
    move |ident: &syn::Ident| format_ident!("{}", converter(&ident.to_string()))
}

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
                    print_warning(
                        "unknown serde attribute",
                        format!(
                            "unknown attribute \"{}\"",
                            meta.path
                                .get_ident()
                                .map(|ident| ident.to_string())
                                .unwrap_or_else(|| "[error]".into())
                        ),
                    )
                    .unwrap();

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
    r#as: Option<syn::Ident>,
    r#type: Option<syn::Expr>,
    crate_path: syn::Path,
}

impl ContainerOptions {
    fn parse(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let mut options = ContainerOptions {
            r#as: None,
            r#type: None,
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
                } else if meta.path.is_ident("as") {
                    options.r#as = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("type") {
                    options.r#type = Some(meta.value()?.parse()?);
                } else {
                    bail!(meta.path, "unknown attribute");
                }

                Ok(())
            })?;
        }

        Ok(options)
    }
}

struct SerdeFieldOptions {
    rename: Option<syn::LitStr>,
}

impl SerdeFieldOptions {
    fn parse(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let mut options = SerdeFieldOptions { rename: None };

        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    options.rename = Some(meta.value()?.parse()?);
                } else {
                    print_warning(
                        "unknown serde attribute",
                        format!(
                            "unknown attribute \"{}\"",
                            meta.path
                                .get_ident()
                                .map(|ident| ident.to_string())
                                .unwrap_or_else(|| "[error]".into())
                        ),
                    )
                    .unwrap();

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

fn patch_type_params<'a, I>(options: &ContainerOptions, type_params: I)
where
    I: Iterator<Item = &'a mut syn::TypeParam>,
{
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

fn collect_struct_fields<'a, C>(
    converter: &'a C,
    crate_path: &'a syn::Path,
    fields: &'a syn::FieldsNamed,
) -> impl Iterator<Item = syn::Result<TokenStream>> + 'a
where
    C: Fn(&syn::Ident) -> syn::Ident,
{
    fields.named.iter().map(move |field| {
        let field_options = SerdeFieldOptions::parse(&field.attrs)?;

        let name = field_options
            .rename
            .map(|lit_str| format_ident!("{}", lit_str.value()))
            .unwrap_or_else(|| converter(field.ident.as_ref().unwrap()));
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
    })
}

fn expand_enum(mut meta: ContainerMeta, input: DataEnum) -> syn::Result<TokenStream> {
    let crate_path = &meta.options.crate_path;
    let converter = ident_adapter(maybe_case_converter(
        meta.serde_options.rename_all.as_ref(),
    )?);

    let mut cases = Vec::new();
    for variant in input.variants.iter() {
        let value = match variant.fields {
            syn::Fields::Named(ref fields) => {
                let items = collect_struct_fields(&converter, crate_path, fields)
                    .collect::<syn::Result<Vec<_>>>()?;

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

        let field_options = SerdeFieldOptions::parse(&variant.attrs)?;

        let variant_name = field_options
            .rename
            .map(|lit_str| format_ident!("{}", lit_str.value()))
            .unwrap_or_else(|| converter(&variant.ident));
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
                    name: stringify!(#name).into(),
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
    let converter = ident_adapter(maybe_case_converter(
        meta.serde_options.rename_all.as_ref(),
    )?);

    let name = &meta.name;
    let description = normalize_option(meta.description.as_ref());
    let crate_path = &meta.options.crate_path;

    let node = if let Some(ref r#as) = meta.options.r#as {
        quote! {
            let definition_resource = #crate_path::Schemaifier::visit_schema(visitor);
            visitor.get_schema::<#r#as>().unwrap().clone()
        }
    } else {
        let node_ty = if let Some(ref r#type) = meta.options.r#type {
            quote! {
                #r#type
            }
        } else {
            let node_ty = match input.fields {
                syn::Fields::Named(ref named) => {
                    let items = collect_struct_fields(&converter, crate_path, named)
                        .collect::<syn::Result<Vec<_>>>()?;

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

            quote! {
                #crate_path::NodeType::Struct(#node_ty)
            }
        };

        quote! {
            #crate_path::Node {
                name: stringify!(#name).into(),
                description: #description,
                value: #node_ty,
            }
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
