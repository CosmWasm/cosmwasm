//! The Cosmwasm IDL (Interface Description Language)

use std::collections::HashMap;

use schemars::schema::RootSchema;
use thiserror::Error;

/// The version of the CosmWasm IDL.
///
/// Follows Semantic Versioning 2.0.0: https://semver.org/
///
/// To determine if a change is breaking, assume consumers allow unknown fields.
pub const IDL_VERSION: &'static str = "0.1.0";

/// Rust representation of a contract's API.
pub struct Api {
    pub instantiate: RootSchema,
    pub execute: RootSchema,
    pub query: RootSchema,
    pub migrate: Option<RootSchema>,
    pub sudo: Option<RootSchema>,
    /// A mapping of query variants to response types
    pub responses: HashMap<String, RootSchema>,
}

impl Api {
    pub fn render(self) -> JsonApi<'static> {
        let mut json_api = JsonApi {
            idl_version: IDL_VERSION,
            instantiate: self.instantiate,
            execute: self.execute,
            query: self.query,
            migrate: self.migrate,
            sudo: self.sudo,
            responses: self.responses,
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
        if let Some(migrate) = &mut json_api.migrate {
            if let Some(metadata) = &mut migrate.schema.metadata {
                metadata.title = Some("MigrateMsg".to_string());
            }
        }
        if let Some(sudo) = &mut json_api.sudo {
            if let Some(metadata) = &mut sudo.schema.metadata {
                metadata.title = Some("SudoMsg".to_string());
            }
        }

        json_api
    }
}

/// A JSON representation of a contract's API.
#[derive(serde::Serialize)]
pub struct JsonApi<'v> {
    idl_version: &'v str,
    instantiate: RootSchema,
    execute: RootSchema,
    query: RootSchema,
    migrate: Option<RootSchema>,
    sudo: Option<RootSchema>,
    responses: HashMap<String, RootSchema>,
}

impl JsonApi<'_> {
    pub fn to_string(&self) -> Result<String, EncodeError> {
        serde_json::to_string_pretty(&self).map_err(Into::into)
    }

    pub fn to_writer(&self, writer: impl std::io::Write) -> Result<(), EncodeError> {
        serde_json::to_writer_pretty(writer, self).map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        semver::Version::parse(IDL_VERSION).unwrap();
    }
}
