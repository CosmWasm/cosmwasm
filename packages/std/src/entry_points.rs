/// This macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the WASM module.
///
/// This macro should be invoced in a global scope, and the argument to the macro
/// should be the name of a rust module that is imported in the invocation scope.
/// The module should export four functions with the following signatures:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, Extern, Env, StdResult, Binary,
/// #     InitResult, HandleResult, QueryResult, MigrateResult,
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
/// # type MigrateMsg = ();
/// pub fn migrate<S: Storage, A: Api, Q: Querier>(
///     deps: &mut Extern<S, A, Q>,
///     _env: Env,
///     msg: MigrateMsg,
/// ) -> MigrateResult {
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
#[macro_export]
macro_rules! entry_points {
    ($contract:ident) => {
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

            #[no_mangle]
            extern "C" fn migrate(env_ptr: u32, msg_ptr: u32) -> u32 {
                do_migrate(
                    &$contract::migrate::<ExternalStorage, ExternalApi, ExternalQuerier>,
                    env_ptr,
                    msg_ptr,
                )
            }

            // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
            // automatically because we `use cosmwasm_std`.
        }
    };
}
