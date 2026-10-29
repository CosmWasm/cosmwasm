use crate::error::bail;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    DeriveInput, MetaNameValue, Token,
};

pub struct Options {
    crate_path: syn::Path,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            crate_path: parse_quote!(::cosmwasm_schema),
        }
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut acc = Self::default();
        let params = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;
        for param in params {
            if param.path.is_ident("crate") {
                let path_as_string: syn::LitStr = syn::parse2(param.value.to_token_stream())?;
                acc.crate_path = path_as_string.parse()?
            } else {
                bail!(param, "unknown option");
            }
        }

        Ok(acc)
    }
}

pub fn cw_serde_impl(options: Options, input: DeriveInput) -> syn::Result<DeriveInput> {
    let crate_path = &options.crate_path;
    let crate_path_displayable = crate_path.to_token_stream();

    let cw_schema_path = format!("{crate_path_displayable}::cw_schema");
    let serde_path = format!("{crate_path_displayable}::serde");
    let schemars_path = format!("{crate_path_displayable}::schemars");

    let mut stream = quote! {
        #[derive(
            #crate_path::serde::Serialize,
            #crate_path::serde::Deserialize,
            ::std::clone::Clone,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            #crate_path::schemars::JsonSchema,
            #crate_path::cw_schema::Schemaifier,
        )]
        #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
        #[serde(crate = #serde_path)]
        #[schemaifier(crate = #cw_schema_path)]
        #[schemars(crate = #schemars_path)]
    };

    match &input.data {
        syn::Data::Struct(..) => (),
        syn::Data::Enum(..) => {
            let rename_all = serde_rename_all(&input.attrs)?;
            if !rename_all.has_serialize {
                if rename_all.has_any {
                    stream.extend(quote! { #[serde(rename_all(serialize = "snake_case"))] });
                } else {
                    stream.extend(quote! { #[serde(rename_all = "snake_case")] });
                }
            }
        }
        syn::Data::Union(..) => bail!(input, "unions are not supported"),
    }

    stream.extend(input.to_token_stream());
    syn::parse2(stream)
}

#[derive(Default)]
struct SerdeRenameAll {
    has_any: bool,
    has_serialize: bool,
}

fn serde_rename_all(attributes: &[syn::Attribute]) -> syn::Result<SerdeRenameAll> {
    let mut rename_all = SerdeRenameAll::default();

    for attribute in attributes
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                rename_all.has_any = true;

                if meta.input.peek(Token![=]) {
                    let value = meta.value()?;
                    let _ = value.parse::<proc_macro2::TokenTree>()?;
                    rename_all.has_serialize = true;
                } else if meta.input.peek(syn::token::Paren) {
                    let params;
                    syn::parenthesized!(params in meta.input);
                    let params = Punctuated::<MetaNameValue, Token![,]>::parse_terminated(&params)?;
                    rename_all.has_serialize |=
                        params.iter().any(|param| param.path.is_ident("serialize"));
                }
            } else if meta.input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in meta.input);
                let _ = content.parse::<proc_macro2::TokenStream>()?;
            } else if meta.input.peek(Token![=]) {
                let value = meta.value()?;
                let _ = value.parse::<proc_macro2::TokenTree>()?;
            }

            Ok(())
        })?;
    }

    Ok(rename_all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn crate_rename() {
        let expanded = cw_serde_impl(
            Options {
                crate_path: parse_quote!(::my_crate::cw_schema),
            },
            parse_quote! {
                pub struct InstantiateMsg {
                    pub verifier: String,
                    pub beneficiary: String,
                }
            },
        )
        .unwrap();

        let expected = parse_quote! {
            #[derive(
                ::my_crate::cw_schema::serde::Serialize,
                ::my_crate::cw_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::my_crate::cw_schema::schemars::JsonSchema,
                ::my_crate::cw_schema::cw_schema::Schemaifier,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(crate = ":: my_crate :: cw_schema::serde")]
            #[schemaifier(crate = ":: my_crate :: cw_schema::cw_schema")]
            #[schemars(crate = ":: my_crate :: cw_schema::schemars")]
            pub struct InstantiateMsg {
                pub verifier: String,
                pub beneficiary: String,
            }
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    fn structs() {
        let expanded = cw_serde_impl(
            Options::default(),
            parse_quote! {
                pub struct InstantiateMsg {
                    pub verifier: String,
                    pub beneficiary: String,
                }
            },
        )
        .unwrap();

        let expected = parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::cosmwasm_schema::cw_schema::Schemaifier,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(crate = ":: cosmwasm_schema::serde")]
            #[schemaifier(crate = ":: cosmwasm_schema::cw_schema")]
            #[schemars(crate = ":: cosmwasm_schema::schemars")]
            pub struct InstantiateMsg {
                pub verifier: String,
                pub beneficiary: String,
            }
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    fn empty_struct() {
        let expanded = cw_serde_impl(
            Options::default(),
            parse_quote! {
                pub struct InstantiateMsg {}
            },
        )
        .unwrap();

        let expected = parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::cosmwasm_schema::cw_schema::Schemaifier,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(crate = ":: cosmwasm_schema::serde")]
            #[schemaifier(crate = ":: cosmwasm_schema::cw_schema")]
            #[schemars(crate = ":: cosmwasm_schema::schemars")]
            pub struct InstantiateMsg {}
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    fn enums() {
        let expanded = cw_serde_impl(
            Options::default(),
            parse_quote! {
                pub enum SudoMsg {
                    StealFunds {
                        recipient: String,
                        amount: Vec<Coin>,
                    },
                }
            },
        )
        .unwrap();

        let expected = parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::cosmwasm_schema::cw_schema::Schemaifier,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(crate = ":: cosmwasm_schema::serde")]
            #[schemaifier(crate = ":: cosmwasm_schema::cw_schema")]
            #[schemars(crate = ":: cosmwasm_schema::schemars")]
            #[serde(rename_all = "snake_case")]
            pub enum SudoMsg {
                StealFunds {
                    recipient: String,
                    amount: Vec<Coin>,
                },
            }
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    #[should_panic(expected = "unions are not supported")]
    fn unions() {
        cw_serde_impl(
            Options::default(),
            parse_quote! {
                pub union SudoMsg {
                    x: u32,
                    y: u32,
                }
            },
        )
        .unwrap();
    }
}
