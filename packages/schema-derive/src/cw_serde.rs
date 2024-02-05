use syn::{parse_quote, DeriveInput};

#[cfg(not(feature = "allow-unknown-fields"))]
pub(crate) fn serde_unknown_fields() -> proc_macro2::TokenStream {
    quote::quote!(#[serde(deny_unknown_fields)])
}

#[cfg(feature = "allow-unknown-fields")]
pub(crate) fn serde_unknown_fields() -> proc_macro2::TokenStream {
    quote::quote!()
}

pub fn cw_serde_impl(input: DeriveInput) -> DeriveInput {
    let unknown_fields = serde_unknown_fields();

    match input.data {
        syn::Data::Struct(_) => parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::cosmwasm_schema::schemars::JsonSchema
            )]
            #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
            #[serde(crate = "::cosmwasm_schema::serde")]
            #unknown_fields
            #[schemars(crate = "::cosmwasm_schema::schemars")]
            #input
        },
        syn::Data::Enum(_) => parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::std::clone::Clone,
                ::std::fmt::Debug,
                ::std::cmp::PartialEq,
                ::cosmwasm_schema::schemars::JsonSchema
            )]
            #[allow(clippy::derive_partial_eq_without_eq)] // Allow users of `#[cw_serde]` to not implement Eq without clippy complaining
            #[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
            #unknown_fields
            #[schemars(crate = "::cosmwasm_schema::schemars")]
            #input
        },
        syn::Data::Union(_) => panic!("unions are not supported"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structs() {
        let expanded = cw_serde_impl(parse_quote! {
            pub struct InstantiateMsg {
                pub verifier: String,
                pub beneficiary: String,
            }
        });

        let unknown_fields = serde_unknown_fields();

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
            #[serde(crate = "::cosmwasm_schema::serde")]
            #unknown_fields
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
        });

        let unknown_fields = serde_unknown_fields();

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
            #[serde(crate = "::cosmwasm_schema::serde")]
            #unknown_fields
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
        });

        let unknown_fields = serde_unknown_fields();

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
            #[serde(rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
            #unknown_fields
            #[schemars(crate = "::cosmwasm_schema::schemars")]
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
        });
    }
}
