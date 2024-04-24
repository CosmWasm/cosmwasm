use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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
                ret.crate_path = path_as_string.parse()?
            } else {
                return Err(syn::Error::new_spanned(kv, "Unknown attribute"));
            }
        }

        Ok(ret)
    }
}

/// This attribute macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the Wasm module.
///
/// It should be added to the contract's init, handle, migrate and query implementations
/// like this:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// #
/// # type InstantiateMsg = ();
/// # type ExecuteMsg = ();
/// # type QueryMsg = ();
///
/// #[entry_point]
/// pub fn instantiate(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn query(
///     deps: Deps,
///     env: Env,
///     msg: QueryMsg,
/// ) -> Result<QueryResponse, StdError> {
/// #   Ok(Default::default())
/// }
/// ```
///
/// where `InstantiateMsg`, `ExecuteMsg`, and `QueryMsg` are contract defined
/// types that implement `DeserializeOwned + JsonSchema`.
///
/// ## Set the version of the state of your contract
///
/// The VM will use this as a hint whether it needs to run the migrate function of your contract or not.
///
/// ```
/// # use cosmwasm_std::{
/// #     DepsMut, entry_point, Env,
/// #     Response, StdResult,
/// # };
/// #
/// # type MigrateMsg = ();
/// #[entry_point]
/// #[set_contract_state_version(version = 2)]
/// pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
///     todo!();
/// }
/// ```
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
        if !attribute.path().is_ident("set_contract_state_version") {
            func.attrs.push(attribute);
            continue;
        }

        if func.sig.ident != "migrate" {
            return Err(syn::Error::new_spanned(
                &attribute,
                "you only want to add this macro to your migrate function",
            ));
        }

        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("version") {
                return Err(meta.error("unexpected attribute"));
            }

            let version: syn::LitInt = meta.value()?.parse()?;
            let version = version.base10_digits();

            stream = quote! {
                #stream

                #[allow(unused)]
                #[doc(hidden)]
                #[link_section = "cw_contract_state_version"]
                /// This is an internal constant exported as a custom section denoting the contract state version.
                /// The format and even the existence of this value is an implementation detail, DO NOT RELY ON THIS!
                static __CW_CONTRACT_STATE_VERSION: &str = #version;
            };

            Ok(())
        })?;
    }

    Ok(stream)
}

fn entry_point_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function: syn::ItemFn = maybe!(syn::parse2(item));
    let Options { crate_path } = maybe!(syn::parse2(attr));

    let attribute_code = maybe!(expand_attributes(&mut function));

    // The first argument is `deps`, the rest is region pointers
    let args = function.sig.inputs.len().saturating_sub(1);
    let fn_name = &function.sig.ident;
    let wasm_export = format_ident!("__wasm_export_{fn_name}");
    let do_call = format_ident!("do_{fn_name}");

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

#[cfg(test)]
mod test {
    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::entry_point_impl;

    #[test]
    fn contract_state_version_expansion() {
        let code = quote! {
            #[set_contract_state_version(version = 2)]
            fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Response {
                // Logic here
            }
        };

        let actual = entry_point_impl(TokenStream::new(), code);
        let expected = quote! {
            #[allow(unused)]
            #[doc(hidden)]
            #[link_section = "cw_contract_state_version"]
            /// This is an internal constant exported as a custom section denoting the contract state version.
            /// The format and even the existence of this value is an implementation detail, DO NOT RELY ON THIS!
            static __CW_CONTRACT_STATE_VERSION: &str = "2";

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
