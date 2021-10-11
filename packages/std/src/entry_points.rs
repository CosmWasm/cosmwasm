/// This macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the Wasm module.
///
/// It should be invoked in a module scope(that is, not inside a function), and the argument to the macro
/// should be the name of a second rust module that is imported in the invocation scope.
/// The second module should export three functions with the following signatures:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Deps, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// #
/// # type InstantiateMsg = ();
/// pub fn instantiate(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// # type ExecuteMsg = ();
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// # type QueryMsg = ();
/// pub fn query(
///     deps: Deps,
///     env: Env,
///     msg: QueryMsg,
/// ) -> Result<QueryResponse, StdError> {
/// #   Ok(Default::default())
/// }
/// ```
/// where `InstantiateMsg`, `ExecuteMsg`, and `QueryMsg` are types that implement `DeserializeOwned + JsonSchema`.
///
/// # Example
///
/// ```ignore
/// use contract; // The contract module
///
/// cosmwasm_std::create_entry_points!(contract);
/// ```
#[macro_export]
#[deprecated(
    note = "create_entry_points and create_entry_points_with_migration should be replaces by the #[entry_point] macro as shown in https://github.com/CosmWasm/cosmwasm/blob/main/MIGRATING.md#013---014. They'll be removed before the final 1.0.0 release. Sorry for the short notice."
)]
macro_rules! create_entry_points {
    (@migration; $contract:ident, true) => {
        #[no_mangle]
        extern "C" fn migrate(env_ptr: u32, msg_ptr: u32) -> u32 {
            do_migrate(&$contract::migrate, env_ptr, msg_ptr)
        }
    };

    (@migration; $contract:ident, false) => {};

    (@inner; $contract:ident, migration = $migration:tt) => {
        mod wasm {
            use super::$contract;
            use cosmwasm_std::{do_execute, do_instantiate, do_migrate, do_query};

            #[no_mangle]
            extern "C" fn instantiate(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
                do_instantiate(&$contract::instantiate, env_ptr, info_ptr, msg_ptr)
            }

            #[no_mangle]
            extern "C" fn execute(env_ptr: u32, info_ptr: u32, msg_ptr: u32) -> u32 {
                do_execute(&$contract::execute, env_ptr, info_ptr, msg_ptr)
            }

            #[no_mangle]
            extern "C" fn query(env_ptr: u32, msg_ptr: u32) -> u32 {
                do_query(&$contract::query, env_ptr, msg_ptr)
            }

            $crate::create_entry_points!(@migration; $contract, $migration);

            // Other C externs like interface_version_8, allocate, deallocate are available
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
/// #     Storage, Api, Querier, DepsMut, Env, StdError, Response, MessageInfo,
/// # };
/// # type MigrateMsg = ();
/// pub fn migrate(
///     deps: DepsMut,
///     _env: Env,
///     msg: MigrateMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
/// ```
/// where `MigrateMsg` is a type that implements `DeserializeOwned + JsonSchema`.
///
/// # Example
///
/// ```ignore
/// use contract; // The contract module
///
/// cosmwasm_std::create_entry_points_with_migration!(contract);
/// ```
#[macro_export]
#[deprecated(
    note = "create_entry_points and create_entry_points_with_migration should be replaces by the #[entry_point] macro as shown in https://github.com/CosmWasm/cosmwasm/blob/main/MIGRATING.md#013---014. They'll be removed before the final 1.0.0 release. Sorry for the short notice."
)]
macro_rules! create_entry_points_with_migration {
    ($contract:ident) => {
        $crate::create_entry_points!(@inner; $contract, migration = true);
    };
}
