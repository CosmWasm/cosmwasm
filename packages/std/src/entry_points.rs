/// This macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the WASM module.
///
/// This macro should be invoced in a global scope, and the argument to the macro
/// should be the name of a rust module that is imported in the invocation scope.
/// The module should export four functions with the following signatures:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, Extern, Env, StdResult, Binary,
/// #     InitResult, HandleResult, QueryResult,
/// # };
///
/// # type InitMsg = ();
/// pub fn init<S: Storage, A: Api, Q: Querier>(
///     deps: &mut Extern<S, A, Q>,
///     env: Env,
///     msg: InitMsg,
/// ) -> InitResult {
/// #   Ok(Default::default())
/// }
///
/// # type HandleMsg = ();
/// pub fn handle<S: Storage, A: Api, Q: Querier>(
///     deps: &mut Extern<S, A, Q>,
///     env: Env,
///     msg: HandleMsg,
/// ) -> HandleResult {
/// #   Ok(Default::default())
/// }
///
/// # type QueryMsg = ();
/// pub fn query<S: Storage, A: Api, Q: Querier>(
///     deps: &Extern<S, A, Q>,
///     msg: QueryMsg,
/// ) -> QueryResult {
/// #   Ok(Binary(Vec::new()))
/// }
/// ```
/// Where `InitMsg`, `HandleMsg`, and `QueryMsg` are types that implement `DeserializeOwned + JsonSchema`
#[macro_export]
macro_rules! entry_points {
    (@migration; $contract:ident, true) => {
        #[no_mangle]
        extern "C" fn migrate(env_ptr: u32, msg_ptr: u32) -> u32 {
            do_migrate(
                &$contract::migrate::<ExternalStorage, ExternalApi, ExternalQuerier>,
                env_ptr,
                msg_ptr,
            )
        }
    };

    (@migration; $contract:ident, false) => {};

    (@inner; $contract:ident, migration = $migration:tt) => {
        mod wasm {
            use super::$contract;
            use cosmwasm_std::{
                do_handle, do_init, do_migrate, do_query, ExternalApi, ExternalQuerier,
                ExternalStorage,
            };

            #[no_mangle]
            extern "C" fn init(env_ptr: u32, msg_ptr: u32) -> u32 {
                do_init(
                    &$contract::init::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    env_ptr,
                    msg_ptr,
                )
            }

            #[no_mangle]
            extern "C" fn handle(env_ptr: u32, msg_ptr: u32) -> u32 {
                do_handle(
                    &$contract::handle::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    env_ptr,
                    msg_ptr,
                )
            }

            #[no_mangle]
            extern "C" fn query(msg_ptr: u32) -> u32 {
                do_query(
                    &$contract::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    msg_ptr,
                )
            }

            $crate::entry_points!(@migration; $contract, $migration);

            // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
            // automatically because we `use cosmwasm_std`.
        }
    };

    ($contract:ident) => {
        $crate::entry_points!(@inner; $contract, migration = false);
    };
}

/// This macro is very similar to the `entry_points` macro, except it also requires the `migrate` method:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, Extern, Env, StdResult, Binary, MigrateResult,
/// # };
/// # type MigrateMsg = ();
/// pub fn migrate<S: Storage, A: Api, Q: Querier>(
///     deps: &mut Extern<S, A, Q>,
///     _env: Env,
///     msg: MigrateMsg,
/// ) -> MigrateResult {
/// #   Ok(Default::default())
/// }
/// ```
/// Where `MigrateMsg` is a type that implements `DeserializeOwned + JsonSchema`
#[macro_export]
macro_rules! entry_points_with_migration {
    ($contract:ident) => {
        $crate::entry_points!(@inner; $contract, migration = true);
    };
}
