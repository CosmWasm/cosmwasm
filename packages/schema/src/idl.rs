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
pub struct CwApi {
    pub contract_name: String,
    pub contract_version: String,
    pub instantiate: Option<cw_schema::Schema>,
    pub execute: Option<cw_schema::Schema>,
    pub query: Option<cw_schema::Schema>,
    pub migrate: Option<cw_schema::Schema>,
    pub sudo: Option<cw_schema::Schema>,
    /// A mapping of query variants to response types
    pub responses: Option<BTreeMap<String, cw_schema::Schema>>,
}

impl CwApi {
    pub fn render(self) -> JsonCwApi {
        JsonCwApi {
            contract_name: self.contract_name,
            contract_version: self.contract_version,
            idl_version: IDL_VERSION.to_string(),
            instantiate: self.instantiate,
            execute: self.execute,
            query: self.query,
            migrate: self.migrate,
            sudo: self.sudo,
            responses: self.responses,
        }
    }
}

/// Rust representation of a contract's API.
pub struct Api {
    pub contract_name: String,
    pub contract_version: String,
    pub instantiate: Option<RootSchema>,
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

        if let Some(instantiate) = &mut json_api.instantiate {
            if let Some(metadata) = &mut instantiate.schema.metadata {
                metadata.title = Some("InstantiateMsg".to_string());
            }
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
#[derive(serde::Deserialize, serde::Serialize)]
pub struct JsonCwApi {
    pub contract_name: String,
    pub contract_version: String,
    pub idl_version: String,
    pub instantiate: Option<cw_schema::Schema>,
    pub execute: Option<cw_schema::Schema>,
    pub query: Option<cw_schema::Schema>,
    pub migrate: Option<cw_schema::Schema>,
    pub sudo: Option<cw_schema::Schema>,
    pub responses: Option<BTreeMap<String, cw_schema::Schema>>,
}

impl JsonCwApi {
    pub fn to_string(&self) -> Result<String, EncodeError> {
        serde_json::to_string_pretty(&self).map_err(Into::into)
    }

    pub fn to_schema_files(&self) -> Result<Vec<(String, String)>, EncodeError> {
        let mut result = Vec::new();

        if let Some(instantiate) = &self.instantiate {
            result.push((
                "instantiate.json".to_string(),
                serde_json::to_string_pretty(&instantiate)?,
            ));
        }

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
                    format!("response_to_{name}.json"),
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

/// A JSON representation of a contract's API.
#[derive(serde::Serialize)]
pub struct JsonApi {
    contract_name: String,
    contract_version: String,
    idl_version: String,
    instantiate: Option<RootSchema>,
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
        let mut result = Vec::new();

        if let Some(instantiate) = &self.instantiate {
            result.push((
                "instantiate.json".to_string(),
                serde_json::to_string_pretty(&instantiate)?,
            ));
        }

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
                    format!("response_to_{name}.json"),
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
    use crate::schema_for;

    use super::*;

    #[test]
    fn version_is_semver() {
        semver::Version::parse(IDL_VERSION).unwrap();
    }

    #[test]
    fn to_schema_files_works() {
        let empty = Api {
            contract_name: "my_contract".to_string(),
            contract_version: "1.2.3".to_string(),
            instantiate: None,
            execute: None,
            query: None,
            migrate: None,
            sudo: None,
            responses: None,
        };

        let files = empty.render().to_schema_files().unwrap();
        assert_eq!(files, []);

        #[derive(schemars::JsonSchema)]
        struct TestMsg {}

        let full = Api {
            contract_name: "my_contract".to_string(),
            contract_version: "1.2.3".to_string(),
            instantiate: Some(schema_for!(TestMsg)),
            execute: Some(schema_for!(TestMsg)),
            query: Some(schema_for!(TestMsg)),
            migrate: Some(schema_for!(TestMsg)),
            sudo: Some(schema_for!(TestMsg)),
            responses: Some(BTreeMap::from([(
                "TestMsg".to_string(),
                schema_for!(TestMsg),
            )])),
        };

        let files = full.render().to_schema_files().unwrap();
        assert_eq!(files.len(), 6);
        assert_eq!(files[0].0, "instantiate.json");
        assert_eq!(files[1].0, "execute.json");
        assert_eq!(files[2].0, "query.json");
        assert_eq!(files[3].0, "migrate.json");
        assert_eq!(files[4].0, "sudo.json");
        assert_eq!(files[5].0, "response_to_TestMsg.json");
    }
}
