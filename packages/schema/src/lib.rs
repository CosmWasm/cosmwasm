mod casing;
mod export;
mod idl;
mod query_response;
mod remove;

pub use export::{export_schema, export_schema_with_title};
pub use idl::{Api, IDL_VERSION};
pub use query_response::{IntegrityError, QueryResponses};
pub use remove::remove_schemas;

// Re-exports
/// An attribute macro that annotates types with things they need to be properly (de)serialized
/// for use in CosmWasm contract messages and/or responses, and also for schema generation.
///
/// This derives things like `serde::Serialize` or `schemars::JsonSchema`, makes sure
/// variants are `snake_case` in the resulting JSON, and so forth.
///
/// # Example
/// ```
/// use cosmwasm_schema::{cw_serde, QueryResponses};
///
/// #[cw_serde]
/// pub struct InstantiateMsg {
///     owner: String,
/// }
///
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// pub enum QueryMsg {
///     #[returns(Vec<String>)]
///     Denoms {},
///     #[returns(String)]
///     AccountName { account: String },
/// }
/// ```
pub use cosmwasm_schema_derive::cw_serde;
/// Generates an [`Api`](crate::Api) for the contract. The body describes the message
/// types exported in the schema and allows setting contract name and version overrides.
///
/// The only obligatory field is `instantiate` - to set the InstantiateMsg type.
///
/// # Available fields
/// See [`write_api`](crate::write_api).
///
/// # Example
/// ```
/// use cosmwasm_schema::{cw_serde, generate_api};
///
/// #[cw_serde]
/// struct InstantiateMsg;
///
/// #[cw_serde]
/// struct MigrateMsg;
///
/// let api = generate_api! {
///     name: "cw20",
///     instantiate: InstantiateMsg,
///     migrate: MigrateMsg,
/// }.render();
/// ```
pub use cosmwasm_schema_derive::generate_api;
/// Takes care of generating the interface description file for a contract. The body describes
/// the message types included and allows setting contract name and version overrides.
///
/// The only obligatory field is `instantiate` - to set the InstantiateMsg type.
///
/// # Available fields
/// - `name` - contract name, crate name by default
/// - `version` - contract version, crate version by default
/// - `instantiate` - instantiate msg type
/// - `query` - query msg type, empty by default
/// - `execute` - execute msg type, empty by default
/// - `migrate` - migrate msg type, empty by default
/// - `sudo` - sudo msg type, empty by default
///
/// # Example
/// ```
/// use cosmwasm_schema::{cw_serde, write_api};
///
/// #[cw_serde]
/// struct InstantiateMsg;
///
/// #[cw_serde]
/// struct MigrateMsg;
///
/// write_api! {
///     name: "cw20",
///     instantiate: InstantiateMsg,
///     migrate: MigrateMsg,
/// };
/// ```
pub use cosmwasm_schema_derive::write_api;
pub use schemars::schema_for;

// For use in macro expansions
pub use schemars;
pub use serde;
