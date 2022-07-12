use syn::{parse_quote, DeriveInput};

pub fn cw_serde_impl(input: DeriveInput) -> DeriveInput {
    match input.data {
        syn::Data::Struct(_) => parse_quote! {
            #[derive(
                serde::Serialize,
                serde::Deserialize,
                Clone,
                Debug,
                PartialEq,
                schemars::JsonSchema
            )]
            #[serde(deny_unknown_fields)]
            #input
        },
        syn::Data::Enum(_) => parse_quote! {
            #[derive(
                serde::Serialize,
                serde::Deserialize,
                Clone,
                Debug,
                PartialEq,
                schemars::JsonSchema
            )]
            #[serde(deny_unknown_fields, rename_all = "snake_case")]
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

        let expected = parse_quote! {
            #[derive(
                serde::Serialize,
                serde::Deserialize,
                Clone,
                Debug,
                PartialEq,
                schemars::JsonSchema
            )]
            #[serde(deny_unknown_fields)]
            pub struct InstantiateMsg {
                pub verifier: String,
                pub beneficiary: String,
            }
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

        let expected = parse_quote! {
            #[derive(
                serde::Serialize,
                serde::Deserialize,
                Clone,
                Debug,
                PartialEq,
                schemars::JsonSchema
            )]
            #[serde(deny_unknown_fields, rename_all = "snake_case")]
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
