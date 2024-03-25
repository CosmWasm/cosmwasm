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
    let serde_path = format!("{crate_path_displayable}::serde");
    let schemars_path = format!("{crate_path_displayable}::schemars");

    let mut stream = quote! {
        #[derive(
            #crate_path::serde::Serialize,
            #crate_path::serde::Deserialize,
            ::std::clone::Clone,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            #crate_path::schemars::JsonSchema
        )]
        #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
        #[serde(deny_unknown_fields, crate = #serde_path)]
        #[schemars(crate = #schemars_path)]
    };

    match input.data {
        syn::Data::Struct(..) => (),
        syn::Data::Enum(..) => {
            stream.extend(quote! { #[serde(rename_all = "snake_case")] });
        }
        syn::Data::Union(..) => bail!(input, "unions are not supported"),
    }

    stream.extend(input.to_token_stream());
    syn::parse2(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

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
                ::cosmwasm_schema::schemars::JsonSchema
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
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
                ::cosmwasm_schema::schemars::JsonSchema
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
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
                ::cosmwasm_schema::schemars::JsonSchema
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
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
