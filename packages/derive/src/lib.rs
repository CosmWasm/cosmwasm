use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Token,
};

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
#[proc_macro_attribute]
pub fn entry_point(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let Options { crate_path } = parse_macro_input!(attr as Options);

    // The first argument is `deps`, the rest is region pointers
    let args = function.sig.inputs.len() - 1;
    let fn_name = function.sig.ident;
    let wasm_export = format_ident!("__wasm_export_{fn_name}");
    let do_call = format_ident!("do_{fn_name}");

    let decl_args = (0..args).map(|item| format_ident!("ptr_{item}"));
    let call_args = decl_args.clone();

    let new_code = quote! {
        #[cfg(target_arch = "wasm32")]
        mod #wasm_export { // new module to avoid conflict of function name
            #[no_mangle]
            extern "C" fn #fn_name(#( #decl_args : u32 ),*) -> u32 {
                #crate_path::#do_call(&super::#fn_name, #( #call_args ),*)
            }
        }
    };

    item.extend(TokenStream::from(new_code));
    item
}
