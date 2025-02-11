use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::env;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    ItemFn, Token,
};

macro_rules! maybe {
    ($result:expr) => {{
        match { $result } {
            Ok(val) => val,
            Err(err) => return err.into_compile_error(),
        }
    }};
}

struct Options {
    crate_path: syn::Path,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            crate_path: parse_quote!(::cosmwasm_std),
        }
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ret = Self::default();
        let attrs = Punctuated::<syn::MetaNameValue, Token![,]>::parse_terminated(input)?;

        for kv in attrs {
            if kv.path.is_ident("crate") {
                let path_as_string: syn::LitStr = syn::parse2(kv.value.to_token_stream())?;
                ret.crate_path = path_as_string.parse()?;
            } else {
                return Err(syn::Error::new_spanned(kv, "Unknown attribute"));
            }
        }

        Ok(ret)
    }
}

// function documented in cosmwasm-std
#[proc_macro_attribute]
pub fn entry_point(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    entry_point_impl(attr.into(), item.into()).into()
}

fn expand_attributes(func: &mut ItemFn) -> syn::Result<TokenStream> {
    let attributes = std::mem::take(&mut func.attrs);
    let mut stream = TokenStream::new();
    for attribute in attributes {
        if !attribute.path().is_ident("migrate_version") {
            func.attrs.push(attribute);
            continue;
        }

        if func.sig.ident != "migrate" {
            return Err(syn::Error::new_spanned(
                &attribute,
                "you only want to add this attribute to your migrate function",
            ));
        }

        let version: syn::Expr = attribute.parse_args()?;
        if !(matches!(version, syn::Expr::Lit(_)) || matches!(version, syn::Expr::Path(_))) {
            return Err(syn::Error::new_spanned(
                &attribute,
                "Expected `u64` or `path::to::constant` in the migrate_version attribute",
            ));
        }

        stream = quote! {
            #stream

            const _: () = {
                #[allow(unused)]
                #[doc(hidden)]
                #[cfg(target_arch = "wasm32")]
                #[link_section = "cw_migrate_version"]
                /// This is an internal constant exported as a custom section denoting the contract migrate version.
                /// The format and even the existence of this value is an implementation detail, DO NOT RELY ON THIS!
                static __CW_MIGRATE_VERSION: [u8; version_size(#version)] = stringify_version(#version);

                #[allow(unused)]
                #[doc(hidden)]
                const fn stringify_version<const N: usize>(mut version: u64) -> [u8; N] {
                    let mut result: [u8; N] = [0; N];
                    let mut index = N;
                    while index > 0 {
                        let digit: u8 = (version%10) as u8;
                        result[index-1] = digit + b'0';
                        version /= 10;
                        index -= 1;
                    }
                    result
                }

                #[allow(unused)]
                #[doc(hidden)]
                const fn version_size(version: u64) -> usize {
                    if version > 0 {
                        (version.ilog10()+1) as usize
                    } else {
                        panic!("Contract migrate version should be greater than 0.")
                    }
                }
            };
        };
    }

    Ok(stream)
}

fn expand_bindings(crate_path: &syn::Path, mut function: syn::ItemFn) -> TokenStream {
    let attribute_code = maybe!(expand_attributes(&mut function));

    // The first argument is `deps`, the rest is region pointers
    let args = function.sig.inputs.len().saturating_sub(1);
    let fn_name = &function.sig.ident;
    let wasm_export = format_ident!("__wasm_export_{fn_name}");

    // Prevent contract dev from using the wrong identifier for the do_migrate_with_info function
    if fn_name == "migrate_with_info" {
        return syn::Error::new_spanned(
            &function.sig.ident,
            r#"To use the new migrate function signature, you should provide a "migrate" entry point with 4 arguments, not "migrate_with_info""#,
        ).into_compile_error();
    }

    // Migrate entry point can take 2 or 3 arguments (not counting deps)
    let do_call = if fn_name == "migrate" && args == 3 {
        format_ident!("do_migrate_with_info")
    } else {
        format_ident!("do_{fn_name}")
    };

    let decl_args = (0..args).map(|item| format_ident!("ptr_{item}"));
    let call_args = decl_args.clone();

    quote! {
        #attribute_code

        #function

        #[cfg(target_arch = "wasm32")]
        mod #wasm_export { // new module to avoid conflict of function name
            #[no_mangle]
            extern "C" fn #fn_name(#( #decl_args : u32 ),*) -> u32 {
                #crate_path::#do_call(&super::#fn_name, #( #call_args ),*)
            }
        }
    }
}

fn entry_point_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function: syn::ItemFn = maybe!(syn::parse2(item));
    let Options { crate_path } = maybe!(syn::parse2(attr));

    if env::var("CARGO_PRIMARY_PACKAGE").is_ok() {
        expand_bindings(&crate_path, function)
    } else {
        function
            .attrs
            .retain(|attr| !attr.path().is_ident("migrate_version"));

        quote! { #function }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::entry_point_impl;

    fn setup_environment() {
        env::set_var("CARGO_PRIMARY_PACKAGE", "1");
    }

    #[test]
    fn contract_migrate_version_on_non_migrate() {
        setup_environment();

        let code = quote! {
            #[migrate_version(42)]
            fn anything_else() -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            ::core::compile_error! { "you only want to add this attribute to your migrate function" }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn contract_migrate_version_expansion() {
        setup_environment();

        let code = quote! {
            #[migrate_version(2)]
            fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            const _: () = {
                #[allow(unused)]
                #[doc(hidden)]
                #[cfg(target_arch = "wasm32")]
                #[link_section = "cw_migrate_version"]
                /// This is an internal constant exported as a custom section denoting the contract migrate version.
                /// The format and even the existence of this value is an implementation detail, DO NOT RELY ON THIS!
                static __CW_MIGRATE_VERSION: [u8; version_size(2)] = stringify_version(2);

                #[allow(unused)]
                #[doc(hidden)]
                const fn stringify_version<const N: usize>(mut version: u64) -> [u8; N] {
                    let mut result: [u8; N] = [0; N];
                    let mut index = N;
                    while index > 0 {
                        let digit: u8 = (version%10) as u8;
                        result[index-1] = digit + b'0';
                        version /= 10;
                        index -= 1;
                    }
                    result
                }

                #[allow(unused)]
                #[doc(hidden)]
                const fn version_size(version: u64) -> usize {
                    if version > 0 {
                        (version.ilog10()+1) as usize
                    } else {
                        panic!("Contract migrate version should be greater than 0.")
                    }
                }
            };

            fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Response {
                // Logic here
            }

            #[cfg(target_arch = "wasm32")]
            mod __wasm_export_migrate {
                #[no_mangle]
                extern "C" fn migrate(ptr_0: u32, ptr_1: u32) -> u32 {
                    ::cosmwasm_std::do_migrate(&super::migrate, ptr_0, ptr_1)
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());

        // this should cause a compiler error
        let code = quote! {
            #[entry_point]
            pub fn migrate_with_info(
                deps: DepsMut,
                env: Env,
                msg: MigrateMsg,
                migrate_info: MigrateInfo,
            ) -> Result<Response, ()> {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            ::core::compile_error! { "To use the new migrate function signature, you should provide a \"migrate\" entry point with 4 arguments, not \"migrate_with_info\"" }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn contract_migrate_version_with_const_expansion() {
        setup_environment();

        let code = quote! {
            #[migrate_version(CONTRACT_VERSION)]
            fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            const _: () = {
                #[allow(unused)]
                #[doc(hidden)]
                #[cfg(target_arch = "wasm32")]
                #[link_section = "cw_migrate_version"]
                /// This is an internal constant exported as a custom section denoting the contract migrate version.
                /// The format and even the existence of this value is an implementation detail, DO NOT RELY ON THIS!
                static __CW_MIGRATE_VERSION: [u8; version_size(CONTRACT_VERSION)] = stringify_version(CONTRACT_VERSION);

                #[allow(unused)]
                #[doc(hidden)]
                const fn stringify_version<const N: usize>(mut version: u64) -> [u8; N] {
                    let mut result: [u8; N] = [0; N];
                    let mut index = N;
                    while index > 0 {
                        let digit: u8 = (version%10) as u8;
                        result[index-1] = digit + b'0';
                        version /= 10;
                        index -= 1;
                    }
                    result
                }

                #[allow(unused)]
                #[doc(hidden)]
                const fn version_size(version: u64) -> usize {
                    if version > 0 {
                        (version.ilog10()+1) as usize
                    } else {
                        panic!("Contract migrate version should be greater than 0.")
                    }
                }
            };

            fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Response {
                // Logic here
            }

            #[cfg(target_arch = "wasm32")]
            mod __wasm_export_migrate {
                #[no_mangle]
                extern "C" fn migrate(ptr_0: u32, ptr_1: u32) -> u32 {
                    ::cosmwasm_std::do_migrate(&super::migrate, ptr_0, ptr_1)
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn default_expansion() {
        setup_environment();

        let code = quote! {
            fn instantiate(deps: DepsMut, env: Env) -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            fn instantiate(deps: DepsMut, env: Env) -> Response { }

            #[cfg(target_arch = "wasm32")]
            mod __wasm_export_instantiate {
                #[no_mangle]
                extern "C" fn instantiate(ptr_0: u32) -> u32 {
                    ::cosmwasm_std::do_instantiate(&super::instantiate, ptr_0)
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn renamed_expansion() {
        setup_environment();

        let attribute = quote!(crate = "::my_crate::cw_std");
        let code = quote! {
            fn instantiate(deps: DepsMut, env: Env) -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(attribute, code);
        let expected = quote! {
            fn instantiate(deps: DepsMut, env: Env) -> Response { }

            #[cfg(target_arch = "wasm32")]
            mod __wasm_export_instantiate {
                #[no_mangle]
                extern "C" fn instantiate(ptr_0: u32) -> u32 {
                    ::my_crate::cw_std::do_instantiate(&super::instantiate, ptr_0)
                }
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
