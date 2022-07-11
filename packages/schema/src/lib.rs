mod casing;
mod export;
mod idl;
mod query_response;
mod remove;

pub use export::{export_schema, export_schema_with_title};
pub use idl::{Api, IDL_VERSION};
pub use query_response::QueryResponses;
pub use remove::remove_schemas;

// Re-exports
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
/// use cosmwasm_schema::{generate_api};
/// use schemars::{JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct InstantiateMsg;
///
/// #[derive(JsonSchema)]
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
/// use cosmwasm_schema::{write_api};
/// use schemars::{JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct InstantiateMsg;
///
/// #[derive(JsonSchema)]
/// struct MigrateMsg;
///
/// generate_api! {
///     name: "cw20",
///     instantiate: InstantiateMsg,
///     migrate: MigrateMsg,
/// };
/// ```
pub use cosmwasm_schema_derive::write_api;
pub use schemars::schema_for;
