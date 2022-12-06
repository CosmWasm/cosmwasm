//! The Cosmwasm IDL (Interface Description Language)

use std::collections::BTreeMap;

use schemars::schema::RootSchema;
use thiserror::Error;

/// The version of the CosmWasm IDL.
///
/// Follows Semantic Versioning 2.0.0: <https://semver.org/>
// To determine if a change is breaking, assume consumers allow unknown fields and bump accordingly.
pub const IDL_VERSION: &str = "1.0.0";

/// Rust representation of a contract's API.
pub struct Api {
    pub contract_name: String,
    pub contract_version: String,
    pub instantiate: RootSchema,
    pub execute: Option<RootSchema>,
    pub query: Option<RootSchema>,
    pub migrate: Option<RootSchema>,
    pub sudo: Option<RootSchema>,
    /// A mapping of query variants to response types
    pub responses: Option<BTreeMap<String, RootSchema>>,
}

impl Api {
    pub fn render(self) -> JsonApi {
        let mut json_api = JsonApi {
            contract_name: self.contract_name,
            contract_version: self.contract_version,
            idl_version: IDL_VERSION.to_string(),
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
        if let Some(execute) = &mut json_api.execute {
            if let Some(metadata) = &mut execute.schema.metadata {
                metadata.title = Some("ExecuteMsg".to_string());
            }
        }
        if let Some(query) = &mut json_api.query {
            if let Some(metadata) = &mut query.schema.metadata {
                metadata.title = Some("QueryMsg".to_string());
            }
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
pub struct JsonApi {
    contract_name: String,
    contract_version: String,
    idl_version: String,
    instantiate: RootSchema,
    execute: Option<RootSchema>,
    query: Option<RootSchema>,
    migrate: Option<RootSchema>,
    sudo: Option<RootSchema>,
    responses: Option<BTreeMap<String, RootSchema>>,
}

impl JsonApi {
    pub fn to_string(&self) -> Result<String, EncodeError> {
        serde_json::to_string_pretty(&self).map_err(Into::into)
    }

    pub fn to_schema_files(&self) -> Result<Vec<(String, String)>, EncodeError> {
        let mut result = vec![(
            "instantiate.json".to_string(),
            serde_json::to_string_pretty(&self.instantiate)?,
        )];

        if let Some(execute) = &self.execute {
            result.push((
                "execute.json".to_string(),
                serde_json::to_string_pretty(&execute)?,
            ));
        }
        if let Some(query) = &self.query {
            result.push((
                "query.json".to_string(),
                serde_json::to_string_pretty(&query)?,
            ));
        }
        if let Some(migrate) = &self.migrate {
            result.push((
                "migrate.json".to_string(),
                serde_json::to_string_pretty(&migrate)?,
            ));
        }
        if let Some(sudo) = &self.sudo {
            result.push((
                "sudo.json".to_string(),
                serde_json::to_string_pretty(&sudo)?,
            ));
        }
        if let Some(responses) = &self.responses {
            for (name, response) in responses {
                result.push((
                    format!("response_to_{}.json", name),
                    serde_json::to_string_pretty(&response)?,
                ));
            }
        }

        Ok(result)
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
