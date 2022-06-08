//! The Cosmwasm IDL (Interface Description Language)

use schemars::schema::RootSchema;

/// The version of the CosmWasm IDL.
///
/// Follows Semantic Versioning 2.0.0: https://semver.org/
///
/// To determine if a change is breaking, assume consumers allow unknown fields.
pub const VERSION: &'static str = "0.1.0";

/// Rust representation of a contract's API.
pub struct Api {
    instantiate: RootSchema,
    execute: RootSchema,
    query: RootSchema,
    response: RootSchema,
}

impl Api {
    fn render(self) -> JsonApi<'static> {
        let mut json_api = JsonApi {
            version: VERSION,
            instantiate: self.instantiate,
            execute: self.execute,
            query: self.query,
            response: self.response,
        };

        if let Some(metadata) = &mut json_api.instantiate.schema.metadata {
            metadata.title = Some("InstantiateMsg".to_string());
        }
        if let Some(metadata) = &mut json_api.execute.schema.metadata {
            metadata.title = Some("ExecuteMsg".to_string());
        }
        if let Some(metadata) = &mut json_api.query.schema.metadata {
            metadata.title = Some("QueryMsg".to_string());
        }
        if let Some(metadata) = &mut json_api.response.schema.metadata {
            metadata.title = Some("QueryResponse".to_string());
        }

        json_api
    }
}

/// A JSON representation of a contract's API.
#[derive(serde::Serialize, serde::Deserialize)]
struct JsonApi<'v> {
    version: &'v str,
    instantiate: RootSchema,
    execute: RootSchema,
    query: RootSchema,
    response: RootSchema,
}
