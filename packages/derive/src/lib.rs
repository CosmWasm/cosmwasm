#[macro_use]
extern crate syn;

mod into_event;

use proc_macro::TokenStream;
use std::str::FromStr;

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
    let name = function.sig.ident.to_string();
    // The first argument is `deps`, the rest is region pointers
    let args = function.sig.inputs.len() - 1;

    // E.g. "ptr0: u32, ptr1: u32, ptr2: u32, "
    let typed_ptrs = (0..args).fold(String::new(), |acc, i| format!("{acc}ptr{i}: u32, "));
    // E.g. "ptr0, ptr1, ptr2, "
    let ptrs = (0..args).fold(String::new(), |acc, i| format!("{acc}ptr{i}, "));

    let new_code = format!(
        r##"
        #[cfg(target_arch = "wasm32")]
        mod __wasm_export_{name} {{ // new module to avoid conflict of function name
            #[no_mangle]
            extern "C" fn {name}({typed_ptrs}) -> u32 {{
                cosmwasm_std::do_{name}(&super::{name}, {ptrs})
            }}
        }}
    "##
    );
    let entry = TokenStream::from_str(&new_code).unwrap();
    item.extend(entry);
    item
}

/// generate an ast for `impl Into<cosmwasm::Event>` from a struct
///
/// Structure:
///
/// ```no_test
/// #[derive(IntoEvent)]
/// struct StructName {
///     field_name_1: field_type_1,
///     // if the value's type does not implement `Into<String>` trait
///     // and it implements `ToString` trait, programmers can specify
///     // to use `field_name_1.to_string()` to get string
///     // by applying `use_to_string`.
///     #[use_to_string]
///     field_name_2: field_type_2,
///     // if the value's type does not implement both `Into<String>` and
///     // `ToString` traits, programmers need specify a function
///     // to get string with `casting_fn(field_name_2)` by applying
///     // `to_string_fn(casting_fn)` attribute.
///     // this `casting_fn` needs to have the type `field_type -> String`.
///     #[to_string_fn(cast_fn_3)]
///     field_name_3: field_type_3,
/// }
/// ```
///
/// Output AST:
///
/// ```no_test
/// impl Into<cosmwasm::Event> for `StructName` {
///     fn into(self) -> Event {
///         Event::new("struct_name")
///             .add_attribute("field_name_1", self.field_value_1)
///             .add_attribute("field_name_2", self.field_value_2.to_string())
///             .add_attribute("field_name_3", casting_fn(self.field_value_3))
///     }
/// }
/// ```
#[proc_macro_derive(IntoEvent, attributes(to_string_fn, use_to_string))]
pub fn derive_into_event(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    into_event::derive_into_event(derive_input)
}
