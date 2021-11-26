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
        Ok(Some(filtered[0].tokens.clone()))
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
            "[IntoEvent] attribute `use_to_string` has some value. If you intend to specify the cast function to string, use `to_string_fn` instead.",
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

/// derive `IntoEvent` from a derive input. The input needs to be a struct.
pub fn derive_into_event(input: DeriveInput) -> TokenStream {
    match input.data {
        syn::Data::Struct(struct_data) => make_init_from_struct(input.ident, struct_data)
            .unwrap_or_else(|e| e.to_compile_error())
            .into(),
        syn::Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Enum.",
        )
        .to_compile_error()
        .into(),
        syn::Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Union.",
        )
        .to_compile_error()
        .into(),
    }
}
