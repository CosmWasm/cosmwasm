use crate::bail;
use owo_colors::{OwoColorize, Style};
use proc_macro2::TokenStream;
use quote::quote;
use std::{
    borrow::Cow,
    collections::BTreeSet,
    env,
    fmt::Display,
    io::{self, Write},
};
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    DataEnum, DataStruct, DataUnion, DeriveInput, Lit,
};

macro_rules! empty_meta {
    ($meta:expr) => {{
        let _ = $meta
            .value()
            .map(|val| val.parse::<TokenStream>().unwrap())
            .unwrap_or_else(|_| $meta.input.cursor().token_stream());
    }};
}

fn print_warning(
    span: proc_macro2::Span,
    title: impl Display,
    content: impl Display,
) -> io::Result<()> {
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

    let span = span.start();
    write!(sink, "{}", "  | ".style(blue))?;
    writeln!(sink, "Location: {}:{}", span.line, span.column)?;

    writeln!(sink, "{}", "  | ".style(blue))?;

    write!(sink, "{}", "  = ".style(blue))?;
    write!(sink, "{}", "note: ".style(bold))?;
    writeln!(
        sink,
        "annotate the container with #[schemaifier(mute_warnings)] to disable these warnings"
    )?;

    Ok(())
}

type Converter = fn(&str) -> String;

fn snake_case_variant(name: &str) -> String {
    let mut renamed = String::new();
    for (index, ch) in name.char_indices() {
        if index > 0 && ch.is_uppercase() {
            renamed.push('_');
        }
        renamed.push(ch.to_ascii_lowercase());
    }

    renamed
}

fn pascal_case_field(name: &str) -> String {
    let mut renamed = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            renamed.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            renamed.push(ch);
        }
    }

    renamed
}

fn camel_case_field(name: &str) -> String {
    let pascal = pascal_case_field(name);
    pascal[..1].to_ascii_lowercase() + &pascal[1..]
}

fn variant_case_converter(case: &syn::LitStr) -> syn::Result<Converter> {
    let converter = match case.value().as_str() {
        "lowercase" => |txt: &str| txt.to_ascii_lowercase(),
        "UPPERCASE" => |txt: &str| txt.to_ascii_uppercase(),
        "PascalCase" => |txt: &str| txt.to_string(),
        "camelCase" => |txt: &str| txt[..1].to_ascii_lowercase() + &txt[1..],
        "snake_case" => snake_case_variant,
        "kebab-case" => |txt: &str| snake_case_variant(txt).replace('_', "-"),
        "SCREAMING_SNAKE_CASE" => |txt: &str| snake_case_variant(txt).to_ascii_uppercase(),
        "SCREAMING-KEBAB-CASE" => |txt: &str| {
            snake_case_variant(txt)
                .to_ascii_uppercase()
                .replace('_', "-")
        },
        _ => return Err(syn::Error::new_spanned(case, "unsupported case style")),
    };

    Ok(converter)
}

fn field_case_converter(case: &syn::LitStr) -> syn::Result<Converter> {
    let converter = match case.value().as_str() {
        "lowercase" | "snake_case" => |txt: &str| txt.to_string(),
        "UPPERCASE" | "SCREAMING_SNAKE_CASE" => |txt: &str| txt.to_ascii_uppercase(),
        "PascalCase" => pascal_case_field,
        "camelCase" => camel_case_field,
        "kebab-case" => |txt: &str| txt.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => |txt: &str| txt.to_ascii_uppercase().replace('_', "-"),
        _ => return Err(syn::Error::new_spanned(case, "unsupported case style")),
    };

    Ok(converter)
}

#[inline]
fn maybe_variant_case_converter(case: Option<&syn::LitStr>) -> syn::Result<Converter> {
    case.map(variant_case_converter)
        .unwrap_or_else(|| Ok(|txt: &str| txt.to_string()))
}

#[inline]
fn maybe_field_case_converter(case: Option<&syn::LitStr>) -> syn::Result<Converter> {
    case.map(field_case_converter)
        .unwrap_or_else(|| Ok(|txt: &str| txt.to_string()))
}

#[inline]
fn renamed_lit(
    rename: Option<syn::LitStr>,
    ident: &syn::Ident,
    converter: Converter,
) -> syn::LitStr {
    rename.unwrap_or_else(|| {
        let name = ident.to_string();
        let name = name.strip_prefix("r#").unwrap_or(&name);
        syn::LitStr::new(&converter(name), ident.span())
    })
}

fn parse_serde_serialize_name(
    meta: syn::meta::ParseNestedMeta,
) -> syn::Result<Option<syn::LitStr>> {
    if meta.input.peek(syn::Token![=]) {
        return Ok(Some(meta.value()?.parse()?));
    }

    let mut serialize_name = None;
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("serialize") {
            serialize_name = Some(nested.value()?.parse()?);
        } else if nested.path.is_ident("deserialize") {
            let _: syn::LitStr = nested.value()?.parse()?;
        } else {
            empty_meta!(nested);
        }

        Ok(())
    })?;

    Ok(serialize_name)
}

struct SerdeContainerOptions {
    rename_all: Option<syn::LitStr>,
    rename_all_fields: Option<syn::LitStr>,
    tag: Option<syn::LitStr>,
    untagged: bool,
}

impl SerdeContainerOptions {
    fn parse(attributes: &[syn::Attribute], muted_warnings: bool) -> syn::Result<Self> {
        let mut options = SerdeContainerOptions {
            rename_all: None,
            rename_all_fields: None,
            tag: None,
            untagged: false,
        };

        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    // ignore the serde crate annotation. we don't really care for that.
                    empty_meta!(meta);
                } else if meta.path.is_ident("rename_all") {
                    if let Some(rename_all) = parse_serde_serialize_name(meta)? {
                        options.rename_all = Some(rename_all);
                    }
                } else if meta.path.is_ident("rename_all_fields") {
                    if let Some(rename_all_fields) = parse_serde_serialize_name(meta)? {
                        options.rename_all_fields = Some(rename_all_fields);
                    }
                } else if meta.path.is_ident("tag") {
                    options.tag = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("untagged") {
                    options.untagged = true;
                } else {
                    if !muted_warnings {
                        print_warning(
                            meta.path.span(),
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
                    }

                    // TODO: support other serde attributes
                    //
                    // See: <https://github.com/CosmWasm/cosmwasm/issues/2499>
                    //
                    // For now we simply clear the buffer to avoid errors
                    empty_meta!(meta);
                }

                Ok(())
            })?;
        }

        if let (true, Some(tag)) = (options.untagged, &options.tag) {
            return Err(syn::Error::new_spanned(
                tag,
                "tag cannot be combined with untagged",
            ));
        }

        Ok(options)
    }
}

struct ContainerOptions {
    r#as: Option<syn::Ident>,
    r#type: Option<syn::Expr>,
    crate_path: syn::Path,
    muted_warnings: bool,
}

impl ContainerOptions {
    fn parse(attributes: &[syn::Attribute]) -> syn::Result<Self> {
        let mut options = ContainerOptions {
            r#as: None,
            r#type: None,
            crate_path: syn::parse_str("::cw_schema")?,
            muted_warnings: false,
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
                } else if meta.path.is_ident("mute_warnings") {
                    options.muted_warnings = true;
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
    default: bool,
    flatten: bool,
    rename: Option<syn::LitStr>,
    skip_serializing_if: Option<syn::Expr>,
    skip: bool,
}

impl SerdeFieldOptions {
    fn parse(attributes: &[syn::Attribute], muted_warnings: bool) -> syn::Result<Self> {
        let mut options = SerdeFieldOptions {
            default: false,
            flatten: false,
            rename: None,
            skip_serializing_if: None,
            skip: false,
        };

        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("serde"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if let Some(rename) = parse_serde_serialize_name(meta)? {
                        options.rename = Some(rename);
                    }
                } else if meta.path.is_ident("default") {
                    options.default = true;
                    if meta.input.peek(syn::Token![=]) {
                        let _: syn::LitStr = meta.value()?.parse()?;
                    }
                } else if meta.path.is_ident("flatten") {
                    options.flatten = true;
                } else if meta.path.is_ident("skip") {
                    options.skip = true;
                } else if meta.path.is_ident("skip_serializing_if") {
                    options.skip_serializing_if = Some(meta.value()?.parse()?);
                } else {
                    if !muted_warnings {
                        print_warning(
                            meta.path.span(),
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
                    }

                    // TODO: support other serde attributes
                    //
                    // See: <https://github.com/CosmWasm/cosmwasm/issues/2499>
                    //
                    // For now we simply clear the buffer to avoid errors
                    empty_meta!(meta);
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

fn type_param_names(generics: &syn::Generics) -> BTreeSet<String> {
    generics
        .type_params()
        .map(|param| param.ident.to_string())
        .collect()
}

struct TypeParamCollector<'a> {
    type_params: &'a BTreeSet<String>,
    used_type_params: &'a mut BTreeSet<String>,
}

impl TypeParamCollector<'_> {
    fn record_type_path(&mut self, path: &syn::Path) {
        if path.leading_colon.is_none() {
            if let Some(segment) = path.segments.first() {
                let ident = segment.ident.to_string();
                if self.type_params.contains(&ident) {
                    self.used_type_params.insert(ident);
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for TypeParamCollector<'_> {
    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        self.record_type_path(&ty.path);
        visit::visit_type_path(self, ty);
    }

    fn visit_expr_path(&mut self, expr: &'ast syn::ExprPath) {
        // Bare expression names are values; qualified paths may use a type.
        if expr.path.segments.len() > 1 {
            self.record_type_path(&expr.path);
        }
        visit::visit_expr_path(self, expr);
    }

    fn visit_macro(&mut self, _mac: &'ast syn::Macro) {
        // Invocation tokens do not reveal the expansion's type dependencies.
        // Macro-based overrides use the caller's explicit bounds.
    }
}

fn collect_type_params_from_type(
    ty: &syn::Type,
    type_params: &BTreeSet<String>,
    used_type_params: &mut BTreeSet<String>,
) {
    TypeParamCollector {
        type_params,
        used_type_params,
    }
    .visit_type(ty);
}

fn collect_type_params_from_expr(
    expr: &syn::Expr,
    type_params: &BTreeSet<String>,
    used_type_params: &mut BTreeSet<String>,
) {
    TypeParamCollector {
        type_params,
        used_type_params,
    }
    .visit_expr(expr);
}

fn collect_type_params_from_struct_fields(
    fields: &syn::Fields,
    muted_warnings: bool,
    type_params: &BTreeSet<String>,
    used_type_params: &mut BTreeSet<String>,
) -> syn::Result<()> {
    match fields {
        syn::Fields::Named(named) => {
            for field in named.named.iter() {
                let field_options = SerdeFieldOptions::parse(&field.attrs, muted_warnings)?;
                if !field_options.skip {
                    collect_type_params_from_type(&field.ty, type_params, used_type_params);
                }
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            for field in unnamed.unnamed.iter() {
                let field_options = SerdeFieldOptions::parse(&field.attrs, muted_warnings)?;
                if !field_options.skip {
                    collect_type_params_from_type(&field.ty, type_params, used_type_params);
                }
            }
        }
        syn::Fields::Unit => {}
    }

    Ok(())
}

fn collect_type_params_from_enum_fields(
    input: &DataEnum,
    muted_warnings: bool,
    type_params: &BTreeSet<String>,
    used_type_params: &mut BTreeSet<String>,
) -> syn::Result<()> {
    for variant in input.variants.iter() {
        let variant_options = SerdeFieldOptions::parse(&variant.attrs, muted_warnings)?;
        if !variant_options.skip {
            collect_type_params_from_struct_fields(
                &variant.fields,
                muted_warnings,
                type_params,
                used_type_params,
            )?;
        }
    }

    Ok(())
}

fn patch_type_params<'a>(
    options: &ContainerOptions,
    type_params: impl Iterator<Item = &'a mut syn::TypeParam>,
    used_type_params: &BTreeSet<String>,
) {
    let crate_path = &options.crate_path;

    for param in type_params {
        if used_type_params.contains(&param.ident.to_string()) {
            param.bounds.push(syn::TypeParamBound::Verbatim(
                quote! { #crate_path::Schemaifier },
            ));
        }
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
    muted_warnings: bool,
) -> impl Iterator<Item = syn::Result<TokenStream>> + 'a
where
    C: Fn(&syn::Ident) -> syn::LitStr,
{
    fields
        .named
        .iter()
        .map(move |field| {
            let field_options = SerdeFieldOptions::parse(&field.attrs, muted_warnings)?;
            if field_options.skip {
                return Ok(None);
            }

            let name = field_options
                .rename
                .unwrap_or_else(|| converter(field.ident.as_ref().unwrap()));
            let description = normalize_option(extract_documentation(&field.attrs)?);
            let field_ty = &field.ty;
            let defaulting = field_options.default;
            let flattening = field_options.flatten;

            let expanded = quote! {
                (
                    #name.into(),
                    #crate_path::StructProperty {
                        defaulting: #defaulting,
                        flattening: #flattening,
                        description: #description,
                        value: <#field_ty as #crate_path::Schemaifier>::visit_schema(visitor),
                    }
                )
            };

            Ok(Some(expanded))
        })
        .filter_map(|item| item.transpose())
}

fn collect_tuple_field_types(
    fields: &syn::FieldsUnnamed,
    muted_warnings: bool,
) -> syn::Result<Vec<&syn::Type>> {
    let mut types = Vec::new();
    for field in fields.unnamed.iter() {
        let field_options = SerdeFieldOptions::parse(&field.attrs, muted_warnings)?;
        if !field_options.skip {
            types.push(&field.ty);
        }
    }

    Ok(types)
}

fn expand_enum(mut meta: ContainerMeta, input: DataEnum) -> syn::Result<TokenStream> {
    let crate_path = &meta.options.crate_path;
    let variant_converter = maybe_variant_case_converter(meta.serde_options.rename_all.as_ref())?;
    let field_converter =
        maybe_field_case_converter(meta.serde_options.rename_all_fields.as_ref())?;
    let convert_variant = |ident: &syn::Ident| renamed_lit(None, ident, variant_converter);
    let convert_field = |ident: &syn::Ident| renamed_lit(None, ident, field_converter);
    let type_params = type_param_names(&meta.generics);
    let mut used_type_params = BTreeSet::new();
    collect_type_params_from_enum_fields(
        &input,
        meta.options.muted_warnings,
        &type_params,
        &mut used_type_params,
    )?;

    let mut cases = Vec::new();
    for variant in input.variants.iter() {
        let field_options = SerdeFieldOptions::parse(&variant.attrs, meta.options.muted_warnings)?;
        if field_options.skip {
            continue;
        }

        let value = match variant.fields {
            syn::Fields::Named(ref fields) => {
                let items = collect_struct_fields(
                    &convert_field,
                    crate_path,
                    fields,
                    meta.options.muted_warnings,
                )
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
                let types = collect_tuple_field_types(fields, meta.options.muted_warnings)?;

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

        let variant_name = field_options
            .rename
            .unwrap_or_else(|| convert_variant(&variant.ident));
        let description = normalize_option(extract_documentation(&variant.attrs)?);

        let expanded = quote! {
            #crate_path::EnumCase {
                description: #description,
                value: #value,
            }
        };

        cases.push(quote! {
            (
                #variant_name.into(),
                #expanded,
            )
        });
    }

    let name = &meta.name;
    let description = normalize_option(meta.description.as_ref());
    let discriminator = normalize_option(meta.serde_options.tag.as_ref());
    let crate_path = &meta.options.crate_path;

    patch_type_params(
        &meta.options,
        meta.generics.type_params_mut(),
        &used_type_params,
    );
    let (impl_generics, ty_generics, where_clause) = meta.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::Schemaifier for #name #ty_generics #where_clause {
            fn visit_schema(visitor: &mut #crate_path::SchemaVisitor) -> #crate_path::DefinitionReference {
                if let Some(reference) = visitor.get_reference::<Self>() {
                    return reference;
                }

                visitor.reserve_spot(Self::id());

                let node = #crate_path::Node {
                    name: std::any::type_name::<Self>()
                        .replace("::", "_")
                        .replace(['<', '>'], "_")
                        .into(),
                    description: #description,
                    value: #crate_path::NodeType::Enum {
                        discriminator: #discriminator,
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
    let converter = maybe_field_case_converter(meta.serde_options.rename_all.as_ref())?;
    let convert_field = |ident: &syn::Ident| renamed_lit(None, ident, converter);

    let name = &meta.name;
    let description = normalize_option(meta.description.as_ref());
    let crate_path = &meta.options.crate_path;
    let type_params = type_param_names(&meta.generics);
    let mut used_type_params = BTreeSet::new();
    if let Some(ref r#type) = meta.options.r#type {
        collect_type_params_from_expr(r#type, &type_params, &mut used_type_params);
    } else if let Some(ref r#as) = meta.options.r#as {
        let ident = r#as.to_string();
        if type_params.contains(&ident) {
            used_type_params.insert(ident);
        }
    } else {
        collect_type_params_from_struct_fields(
            &input.fields,
            meta.options.muted_warnings,
            &type_params,
            &mut used_type_params,
        )?;
    }

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
                    let items = collect_struct_fields(
                        &convert_field,
                        crate_path,
                        named,
                        meta.options.muted_warnings,
                    )
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
                    let type_names =
                        collect_tuple_field_types(&fields, meta.options.muted_warnings)?;

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
                name: std::any::type_name::<Self>()
                    .replace("::", "_")
                    .replace(['<', '>'], "_")
                    .into(),
                description: #description,
                value: #node_ty,
            }
        }
    };

    patch_type_params(
        &meta.options,
        meta.generics.type_params_mut(),
        &used_type_params,
    );
    let (impl_generics, ty_generics, where_clause) = meta.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::Schemaifier for #name #ty_generics #where_clause {
            fn visit_schema(visitor: &mut #crate_path::SchemaVisitor) -> #crate_path::DefinitionReference {
                if let Some(reference) = visitor.get_reference::<Self>() {
                    return reference;
                }

                visitor.reserve_spot(Self::id());

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
    let serde_options = SerdeContainerOptions::parse(&input.attrs, options.muted_warnings)?;
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
