use quote::quote;
use syn::{parse_quote, DeriveInput};

/// This is only needed for types that do not implement cw_serde.
pub fn cw_prost_impl(input: DeriveInput) -> DeriveInput {
    match input.data {
        syn::Data::Struct(_) => parse_quote! {
            #[derive(
                ::prost::Message,
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #input
        },
        syn::Data::Enum(_) => parse_quote! {
            #[derive(
                ::prost::Oneof,
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #input
        },
        syn::Data::Union(_) => panic!("unions are not supported"),
    }
}

/// You cannot derive both cw_serde and cw_prost on the same type.
/// Use this instead if you want both
pub fn cw_prost_serde_impl(input: DeriveInput) -> DeriveInput {
    match input.data {
        syn::Data::Struct(_) => parse_quote! {
            #[derive(
                ::prost::Message,
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::std::clone::Clone,
                ::std::cmp::PartialEq
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
            #input
        },
        syn::Data::Enum(_) => parse_quote! {
            #[derive(
                ::prost::Oneof,
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::std::clone::Clone,
                ::std::cmp::PartialEq
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, rename_all = "snake_case", crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
            #input
        },
        syn::Data::Union(_) => panic!("unions are not supported"),
    }
}

/// You cannot derive both cw_serde and cw_prost on the same type.
/// Use this instead if you want both
pub fn cw_prost_serde_newtype_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    match &input.data {
        syn::Data::Struct(_) => {}
        syn::Data::Enum(_) => panic!("enums are not supported"),
        syn::Data::Union(_) => panic!("unions are not supported"),
    };

    let ident = input.ident.clone();

    let body: proc_macro2::TokenStream = parse_quote! {
        #[derive(
            ::cosmwasm_schema::serde::Serialize,
            ::cosmwasm_schema::serde::Deserialize,
            ::cosmwasm_schema::schemars::JsonSchema,
            ::std::fmt::Debug,
            ::std::default::Default,
            ::std::clone::Clone,
            ::std::cmp::PartialEq
        )]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
        #[schemars(crate = "::cosmwasm_schema::schemars")]
        #input
    };

    let imp: proc_macro2::TokenStream = parse_quote! {
        impl ::prost::Message for #ident {
            fn encode_raw<B: ::prost::bytes::BufMut>(&self, buf: &mut B) {
                self.0.encode_raw(buf)
            }

            fn clear(&mut self) {
                self.0.clear()
            }

            #[inline]
            fn encoded_len(&self) -> usize {
                self.0.encoded_len()
            }

            fn merge_field<B: ::prost::bytes::Buf>(
                &mut self,
                tag: u32,
                wire_type: ::prost::encoding::WireType,
                buf: &mut B,
                ctx: ::prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), ::prost::DecodeError> {
                self.0.merge_field(tag, wire_type, buf, ctx)
            }
        }
    };

    quote! {
        #body
        #imp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structs() {
        let expanded = cw_prost_impl(parse_quote! {
            pub struct InstantiateMsg {
                #[prost(string, tag="1")]
                pub verifier: String,
                #[prost(string, tag="2")]
                pub beneficiary: String,
            }
        });

        let expected = parse_quote! {
            #[derive(
                ::prost::Message,
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            pub struct InstantiateMsg {
                #[prost(string, tag="1")]
                pub verifier: String,
                #[prost(string, tag="2")]
                pub beneficiary: String,
            }
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    fn newtype() {
        let expanded = cw_prost_serde_newtype_impl(parse_quote! {
            pub struct Addr(String);
        });

        let body: proc_macro2::TokenStream = parse_quote! {
            #[derive(
                ::cosmwasm_schema::serde::Serialize,
                ::cosmwasm_schema::serde::Deserialize,
                ::cosmwasm_schema::schemars::JsonSchema,
                ::std::fmt::Debug,
                ::std::default::Default,
                ::std::clone::Clone,
                ::std::cmp::PartialEq
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[serde(deny_unknown_fields, crate = "::cosmwasm_schema::serde")]
            #[schemars(crate = "::cosmwasm_schema::schemars")]
            pub struct Addr(String);
        };

        let imp: proc_macro2::TokenStream = parse_quote! {
            impl ::prost::Message for Addr {
                fn encode_raw<B: ::prost::bytes::BufMut>(&self, buf: &mut B) {
                    self.0.encode_raw(buf)
                }

                fn clear(&mut self) {
                    self.0.clear()
                }

                #[inline]
                fn encoded_len(&self) -> usize {
                    self.0.encoded_len()
                }

                fn merge_field<B: ::prost::bytes::Buf>(
                    &mut self,
                    tag: u32,
                    wire_type: ::prost::encoding::WireType,
                    buf: &mut B,
                    ctx: ::prost::encoding::DecodeContext,
                ) -> ::core::result::Result<(), ::prost::DecodeError> {
                    self.0.merge_field(tag, wire_type, buf, ctx)
                }
            }
        };

        let expected = quote! {
            #body
            #imp
        };

        println!("{}", expanded);
        println!("{}", expected);
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    #[test]
    fn empty_struct() {
        let expanded = cw_prost_impl(parse_quote! {
            pub struct InstantiateMsg {}
        });

        let expected = parse_quote! {
            #[derive(
                ::prost::Message,
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            pub struct InstantiateMsg {}
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    fn enums() {
        let expanded = cw_prost_impl(parse_quote! {
            pub enum SudoMsg {
                #[prost(message, tag = "1")]
                StealFunds {
                    #[prost(string, tag = "1")]
                    recipient: String,
                    #[prost(message, repeated, tag = "2")]
                    amount: Vec<Coin>,
                },
            }
        });

        let expected = parse_quote! {
            #[derive(
                ::prost::Oneof,
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
            )]
            #[allow(clippy::derive_partial_eq_without_eq)]
            pub enum SudoMsg {
                #[prost(message, tag = "1")]
                StealFunds {
                    #[prost(string, tag = "1")]
                    recipient: String,
                    #[prost(message, repeated, tag = "2")]
                    amount: Vec<Coin>,
                },
            }
        };

        assert_eq!(expanded, expected);
    }

    #[test]
    #[should_panic(expected = "unions are not supported")]
    fn unions() {
        cw_prost_impl(parse_quote! {
            pub union SudoMsg {
                x: u32,
                y: u32,
            }
        });
    }

    #[test]
    #[should_panic(expected = "expected one of: `struct`, `enum`, `union`")]
    fn functions() {
        cw_prost_impl(parse_quote! {
            pub fn do_stuff(a: i32) -> i32 {
                a * 3
            }
        });
    }
}
