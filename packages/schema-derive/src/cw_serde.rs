use manyhow::bail;
use quote::{quote, ToTokens};
use syn::DeriveInput;

pub fn cw_serde_impl(input: DeriveInput) -> syn::Result<DeriveInput> {
    let mut stream = quote! {
        #[derive(
            ::cosmwasm_schema::serde::Serialize,
            ::cosmwasm_schema::serde::Deserialize,
            ::std::clone::Clone,
            ::std::fmt::Debug,
            ::std::cmp::PartialEq,
            ::cosmwasm_schema::schemars::JsonSchema
        )]
        #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
        #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
        #[schemars(crate = "::cosmwasm_schema::schemars")]
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
        let expanded = cw_serde_impl(parse_quote! {
            pub struct InstantiateMsg {
                pub verifier: String,
                pub beneficiary: String,
            }
        })
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
        let expanded = cw_serde_impl(parse_quote! {
            pub struct InstantiateMsg {}
        })
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
        let expanded = cw_serde_impl(parse_quote! {
            pub enum SudoMsg {
                StealFunds {
                    recipient: String,
                    amount: Vec<Coin>,
                },
            }
        })
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
        cw_serde_impl(parse_quote! {
            pub union SudoMsg {
                x: u32,
                y: u32,
            }
        })
        .unwrap();
    }
}
