/// This macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the Wasm module.
///
/// It should be invoked in a module scope(that is, not inside a function), and the argument to the macro
/// should be the name of a second rust module that is imported in the invocation scope.
/// The second module should export three functions with the following signatures:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Deps, Env, StdResult, Binary, MessageInfo,
/// #     InitResult, HandleResult, QueryResult,
/// # };
/// #
/// # type InitMsg = ();
/// pub fn init(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: InitMsg,
/// ) -> InitResult {
/// #   Ok(Default::default())
/// }
///
/// # type HandleMsg = ();
/// pub fn handle(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: HandleMsg,
/// ) -> HandleResult {
/// #   Ok(Default::default())
/// }
///
/// # type QueryMsg = ();
/// pub fn query(
///     deps: Deps,
///     env: Env,
///     msg: QueryMsg,
/// ) -> QueryResult {
/// #   Ok(Binary(Vec::new()))
/// }
/// ```
/// Where `InitMsg`, `HandleMsg`, and `QueryMsg` are types that implement `DeserializeOwned + JsonSchema`
///
/// # Example
///
/// ```ignore
/// use contract; // The contract module
///
/// cosmwasm_std::create_entry_points!(contract);
/// ```
#[macro_export]
macro_rules! create_entry_points {
    (@migration; $contract:ident, true) => {
        #[no_mangle]
        extern "C" fn migrate(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
            do_migrate(&$contract::migrate, env_ptr, info_ptr, msg_ptr)
        }
    };

    (@migration; $contract:ident, false) => {};

    (@inner; $contract:ident, migration = $migration:tt) => {
        mod wasm {
            use super::$contract;
            use cosmwasm_std::{do_handle, do_init, do_migrate, do_query};

            #[no_mangle]
            extern "C" fn init(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
                do_init(&$contract::init, env_ptr, info_ptr, msg_ptr)
            }

            #[no_mangle]
            extern "C" fn handle(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
                do_handle(&$contract::handle, env_ptr, info_ptr, msg_ptr)
            }

            #[no_mangle]
            extern "C" fn query(env_ptr: u32, msg_ptr: u32) -> u32 {
                do_query(&$contract::query, env_ptr, msg_ptr)
            }

            $crate::create_entry_points!(@migration; $contract, $migration);

            // Other C externs like cosmwasm_vm_version_4, allocate, deallocate are available
            // automatically because we `use cosmwasm_std`.
        }
    };

    ($contract:ident) => {
        $crate::create_entry_points!(@inner; $contract, migration = false);
    };
}

/// This macro is very similar to the `create_entry_points` macro, except it also requires the `migrate` method:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Env, StdResult, Binary, MigrateResult, MessageInfo,
/// # };
/// # type MigrateMsg = ();
/// pub fn migrate(
///     deps: DepsMut,
///     _env: Env,
///     _info: MessageInfo,
///     msg: MigrateMsg,
/// ) -> MigrateResult {
/// #   Ok(Default::default())
/// }
/// ```
/// Where `MigrateMsg` is a type that implements `DeserializeOwned + JsonSchema`
///
/// # Example
///
/// ```ignore
/// use contract; // The contract module
///
/// cosmwasm_std::create_entry_points_with_migration!(contract);
/// ```
#[macro_export]
macro_rules! create_entry_points_with_migration {
    ($contract:ident) => {
        $crate::create_entry_points!(@inner; $contract, migration = true);
    };
}
