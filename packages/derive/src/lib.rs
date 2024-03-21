use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

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
pub fn entry_point(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);

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
                cosmwasm_std::#do_call(&super::#fn_name, #( #call_args ),*)
            }
        }
    };

    item.extend::<TokenStream>(new_code.into());
    item
}
