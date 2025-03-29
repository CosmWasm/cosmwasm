use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::prelude::*;

/// Replicates the cosmos-sdk bank module Metadata type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomMetadata {
    pub description: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub denom_units: Vec<DenomUnit>,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub uri_hash: String,
}

/// Replicates the cosmos-sdk bank module DenomUnit type
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomUnit {
    pub denom: String,
    pub exponent: u32,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub aliases: Vec<String>,
}

// Deserialize a field that is null, defaulting to the type's default value.
// Panic if the field is missing.
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use serde_json::{json, Error};

    #[test]
    fn deserialize_denom_metadata_with_null_fields_works() {
        // Test case with null denom_units - should deserialize as empty vec
        let json_with_null_denom_units = json!({
            "description": "Test Token",
            "denom_units": null,
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_null_denom_units: DenomMetadata =
            serde_json::from_value(json_with_null_denom_units).unwrap();
        assert_eq!(
            metadata_null_denom_units.denom_units,
            Vec::<DenomUnit>::default()
        );
        assert!(metadata_null_denom_units.denom_units.is_empty());

        // Test normal case with provided denom_units
        let json_with_units = json!({
            "description": "Test Token",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6,
                    "aliases": ["microtest"]
                }
            ],
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_with_units: DenomMetadata = serde_json::from_value(json_with_units).unwrap();
        assert_eq!(metadata_with_units.denom_units.len(), 1);
        assert_eq!(metadata_with_units.denom_units[0].denom, "utest");

        // Test with null aliases inside denom_units - should deserialize as empty vec
        let json_with_null_aliases = json!({
            "description": "Test Token",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6,
                    "aliases": null
                }
            ],
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_with_null_aliases: DenomMetadata =
            serde_json::from_value(json_with_null_aliases).unwrap();
        assert_eq!(metadata_with_null_aliases.denom_units.len(), 1);
        assert_eq!(
            metadata_with_null_aliases.denom_units[0].aliases,
            Vec::<String>::default()
        );
        assert!(metadata_with_null_aliases.denom_units[0].aliases.is_empty());
    }

    #[test]
    fn deserialize_denom_metadata_with_missing_fields_fails() {
        // Missing denom_units should be treated like null
        let json_missing_denom_units = json!({
            "description": "Test Token",
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata: Result<DenomMetadata, Error> =
            serde_json::from_value(json_missing_denom_units);
        assert!(metadata.is_err());

        let json_missing_alias = json!({
            "description": "Test Token",
            "base": "utest",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6,
                }
            ],
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_missing_alias: Result<DenomMetadata, Error> =
            serde_json::from_value(json_missing_alias);
        assert!(metadata_missing_alias.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::{DenomMetadata, DenomUnit};
    use serde_json::{json, Error};

    #[test]
    fn query_denom_metadata_with_null_denom_units_works() {
        // Test case with null denom_units - should deserialize as empty vec
        let json_with_null_denom_units = json!({
            "description": "Test Token",
            "denom_units": null,
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_with_null_denom_units: DenomMetadata =
            serde_json::from_value(json_with_null_denom_units).unwrap();
        assert_eq!(
            metadata_with_null_denom_units.denom_units,
            Vec::<DenomUnit>::default()
        );
        assert!(metadata_with_null_denom_units.denom_units.is_empty());

        // Test normal case with provided denom_units
        let json_with_units = json!({
            "description": "Test Token",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6,
                    "aliases": ["microtest"]
                }
            ],
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_with_units: DenomMetadata = serde_json::from_value(json_with_units).unwrap();
        assert_eq!(metadata_with_units.denom_units.len(), 1);
        assert_eq!(metadata_with_units.denom_units[0].denom, "utest");
        assert_eq!(metadata_with_units.denom_units[0].aliases.len(), 1);
        assert_eq!(metadata_with_units.denom_units[0].aliases[0], "microtest");

        // Test with null aliases inside denom_units - should deserialize as empty vec
        let json_with_null_aliases = json!({
            "description": "Test Token",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6,
                    "aliases": null
                }
            ],
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let metadata_with_null_aliases: DenomMetadata =
            serde_json::from_value(json_with_null_aliases).unwrap();
        assert_eq!(metadata_with_null_aliases.denom_units.len(), 1);
        assert_eq!(
            metadata_with_null_aliases.denom_units[0].aliases,
            Vec::<String>::default()
        );
        assert!(metadata_with_null_aliases.denom_units[0].aliases.is_empty());
    }

    #[test]
    fn query_denom_metadata_with_missing_fields_fails() {
        // Missing denom_units should be treated as default value (empty vec)
        let json_missing_denom_units = json!({
            "description": "Test Token",
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let json_missing_denom_units_metadata: Result<DenomMetadata, Error> =
            serde_json::from_value(json_missing_denom_units);
        assert!(json_missing_denom_units_metadata.is_err());

        // Missing aliases field should be treated as default (empty vec)
        let json_missing_aliases = json!({
            "description": "Test Token",
            "denom_units": [
                {
                    "denom": "utest",
                    "exponent": 6
                }
            ],
            "base": "utest",
            "display": "TEST",
            "name": "Test Token",
            "symbol": "TEST",
            "uri": "https://test.com",
            "uri_hash": "hash"
        });

        let missing_aliases_metadata: Result<DenomMetadata, Error> =
            serde_json::from_value(json_missing_aliases);
        assert!(missing_aliases_metadata.is_err());
    }

    #[test]
    fn query_denom_metadata_with_mixed_null_and_value_works() {
        // Test with multiple denom units, some with null aliases and some with values
        let mixed_json = json!({
            "description": "Mixed Token",
            "denom_units": [
                {
                    "denom": "unit1",
                    "exponent": 0,
                    "aliases": null
                },
                {
                    "denom": "unit2",
                    "exponent": 6,
                    "aliases": ["microunit", "u"]
                },
                {
                    "denom": "unit3",
                    "exponent": 9,
                    "aliases": []
                }
            ],
            "base": "unit1",
            "display": "MIXED",
            "name": "Mixed Token",
            "symbol": "MIX",
            "uri": "https://mixed.token",
            "uri_hash": "hash123"
        });

        let metadata: DenomMetadata = serde_json::from_value(mixed_json).unwrap();

        // First denom unit has null aliases, should be empty vec
        assert!(metadata.denom_units[0].aliases.is_empty());

        // Second has two aliases
        assert_eq!(metadata.denom_units[1].aliases.len(), 2);
        assert_eq!(metadata.denom_units[1].aliases[0], "microunit");
        assert_eq!(metadata.denom_units[1].aliases[1], "u");

        // Third has explicitly empty aliases
        assert!(metadata.denom_units[2].aliases.is_empty());
    }
}
