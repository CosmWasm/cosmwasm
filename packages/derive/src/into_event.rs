extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DataStruct, DeriveInput, Error, Field, Ident, Result};

/// scan attrs and get `to_string_fn` attribute
fn scan_to_string_fn(field: &Field) -> Result<Option<proc_macro2::TokenStream>> {
    let filtered: Vec<&Attribute> = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("to_string_fn"))
        .collect();
    if filtered.len() > 1 {
        return Err(Error::new(
            field.span(),
            "[IntoEvent] Only one or zero `to_string_fn` can be applied to one field.",
        ));
    };
    if filtered.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filtered[0].parse_args()?))
    }
}

/// scan attrs and return if it has any `to_string`
fn has_use_to_string(field: &Field) -> Result<bool> {
    let mut filtered = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("use_to_string"));
    if filtered.clone().any(|a| !a.tokens.is_empty()) {
        return Err(Error::new(
            field.span(),
            "[IntoEvent] An attribute `use_to_string` has some value. If you intend to specify the cast function to string, use `to_string_fn` instead.",
        ));
    }
    Ok(filtered.next().is_some())
}

/// generate an ast for `impl Into<cosmwasm::Event>` from a struct
fn make_init_from_struct(id: Ident, struct_data: DataStruct) -> Result<proc_macro2::TokenStream> {
    // snake case of struct ident
    let name = id.to_string().as_str().to_case(Case::Snake);

    // generate the body of `fn into`
    // generating `Event::new()` part
    let mut fn_body = quote!(
        cosmwasm_std::Event::new(#name)
    );

    // chain `.add_attribute`s to `Event::new()` part
    for field in struct_data.fields {
        let field_id = match field.clone().ident {
            None => {
                return Err(Error::new(
                    field.span(),
                    "[IntoEvent] Unexpected unnamed field.",
                ))
            }
            Some(field_id) => field_id,
        };
        let value = match (scan_to_string_fn(&field)?, has_use_to_string(&field)?) {
            (Some(_), true) => return Err(Error::new(
                field.span(),
                "[IntoEvent] Both `use_to_string` and `to_string_fn` are applied to an field. Only one can be applied.",
            )),
            (Some(to_string_fn), false) => quote!(#to_string_fn(self.#field_id)),
            (None, true) => quote!(self.#field_id.to_string()),
            (None, false) => quote!(self.#field_id),
        };
        fn_body.extend(quote!(
            .add_attribute(stringify!(#field_id), #value)
        ))
    }

    // generate the `impl Into<cosmwasm_std::Event>` from generated `fn_body`
    let gen = quote!(
        impl Into<cosmwasm_std::Event> for #id {
            fn into(self) -> cosmwasm_std::Event {
                #fn_body
            }
        }
    );
    Ok(gen)
}

fn derive_into_event_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    match input.data {
        syn::Data::Struct(struct_data) => {
            make_init_from_struct(input.ident, struct_data).unwrap_or_else(|e| e.to_compile_error())
        }
        syn::Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Enum.",
        )
        .to_compile_error(),
        syn::Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Union.",
        )
        .to_compile_error(),
    }
}

/// derive `IntoEvent` from a derive input. The input needs to be a struct.
pub fn derive_into_event(input: DeriveInput) -> TokenStream {
    derive_into_event_impl(input).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::parse_quote;

    fn expect_compile_error(ts: TokenStream, msg: &str) {
        assert!(
            ts.to_string().starts_with("compile_error ! {"),
            "Code does not raise compile error: `{}`",
            ts
        );

        assert!(
            ts.to_string().contains(msg),
            "Error does not have expected message \"{}\": `{}`",
            msg,
            ts
        );
    }

    #[test]
    fn test_doc_example() {
        let input: DeriveInput = parse_quote! {
            struct StructName {
                field_name_1: field_type_1,
                #[use_to_string]
                field_name_2: field_type_2,
                #[to_string_fn(cast_fn_3)]
                field_name_3: field_type_3,
            }
        };
        let result_implement = derive_into_event_impl(input);
        let expected: TokenStream = parse_quote! {
            impl Into<cosmwasm_std::Event> for StructName {
                fn into(self) -> cosmwasm_std::Event {
                    cosmwasm_std::Event::new("struct_name")
                        .add_attribute(stringify!(field_name_1), self.field_name_1)
                        .add_attribute(stringify!(field_name_2), self.field_name_2.to_string())
                        .add_attribute(stringify!(field_name_3), cast_fn_3 (self.field_name_3))
                }
            }
        };
        assert_eq!(expected.to_string(), result_implement.to_string())
    }

    #[test]
    fn test_error_multiple_to_string_functions() {
        let input: DeriveInput = parse_quote! {
            struct StructName {
                #[to_string_fn(cast_fn_1)]
                #[to_string_fn(cast_fn_1)]
                field_name_1: field_type_1,
            }
        };
        let result_implement = derive_into_event_impl(input);
        expect_compile_error(
            result_implement,
            "[IntoEvent] Only one or zero `to_string_fn`",
        );
    }

    #[test]
    fn test_error_use_to_string_has_value() {
        let input: DeriveInput = parse_quote! {
            struct StructName {
                #[use_to_string(foo)]
                field_name_1: field_type_1,
            }
        };
        let result_implement = derive_into_event_impl(input);
        expect_compile_error(
            result_implement,
            "[IntoEvent] An attribute `use_to_string` has some value",
        );
    }

    #[test]
    fn test_error_both_two_attributes_is_used() {
        let input: DeriveInput = parse_quote! {
            struct StructName {
                #[use_to_string]
                #[to_string_fn(cast_fn_1)]
                field_name_1: field_type_1,
            }
        };
        let result_implement = derive_into_event_impl(input);
        expect_compile_error(
            result_implement,
            "[IntoEvent] Both `use_to_string` and `to_string_fn`",
        );
    }

    #[test]
    fn test_error_derive_enum() {
        let input: DeriveInput = parse_quote! {
            enum Enum {}
        };
        let result_implement = derive_into_event_impl(input);
        expect_compile_error(
            result_implement,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Enum",
        );
    }

    #[test]
    fn test_error_derive_union() {
        let input: DeriveInput = parse_quote! {
            union Union {}
        };
        let result_implement = derive_into_event_impl(input);
        expect_compile_error(
            result_implement,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Union",
        );
    }
}
