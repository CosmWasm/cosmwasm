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
/// Generates an [`Api`](schema::Api) for the contract. The body describes the message
/// types exported in the schema and allows setting contract name and version overrides.
///
/// The only obligatory field is `instantiate` - to set the InstantiateMsg type.
///
/// # Available fields
/// - `name` - contract name
/// - `version` - contract version
/// - `instantiate` - instantiate msg type
/// - `query` - query msg type
/// - `execute` - execute msg type
/// - `migrate` - migrate msg type
/// - `sudo` - sudo msg type
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
/// generate_api! {
///     name: "cw20",
///     instantiate: InstantiateMsg,
///     migrate: MigrateMsg,
/// };
/// ```
pub use cosmwasm_schema_derive::generate_api;
/// Generates an [`Api`](schema::Api) for the contract. The body describes the message
/// types exported in the schema and allows setting contract name and version overrides.
///
/// The only obligatory field is `instantiate` - to set the InstantiateMsg type.
///
/// # Available fields
/// - `name` - contract name
/// - `version` - contract version
/// - `instantiate` - instantiate msg type
/// - `query` - query msg type
/// - `execute` - execute msg type
/// - `migrate` - migrate msg type
/// - `sudo` - sudo msg type
///
/// # Example
/// ```
/// use cosmwasm_schema::{generate_api_obj};
/// use schemars::{JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct InstantiateMsg;
///
/// #[derive(JsonSchema)]
/// struct MigrateMsg;
///
/// let api = generate_api_obj! {
///     name: "cw20",
///     instantiate: InstantiateMsg,
///     migrate: MigrateMsg,
/// }.render();
/// ```
pub use cosmwasm_schema_derive::generate_api_obj;
pub use schemars::schema_for;
