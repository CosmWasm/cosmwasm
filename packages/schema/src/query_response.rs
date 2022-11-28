use std::collections::{BTreeMap, BTreeSet};

use schemars::{
    schema::{InstanceType, RootSchema, SingleOrVec, SubschemaValidation},
    JsonSchema,
};
use thiserror::Error;

pub use cosmwasm_schema_derive::QueryResponses;

/// A trait for tying QueryMsg variants (different contract queries) to their response types.
/// This is mostly useful for the generated contracted API description when using `cargo schema`.
///
/// Using the derive macro is the preferred way of implementing this trait.
///
/// # Examples
/// ```
/// use cosmwasm_schema::QueryResponses;
/// use schemars::JsonSchema;
///
/// #[derive(JsonSchema)]
/// struct AccountInfo {
///     IcqHandle: String,
/// }
///
/// #[derive(JsonSchema, QueryResponses)]
/// enum QueryMsg {
///     #[returns(Vec<String>)]
///     Denoms {},
///     #[returns(AccountInfo)]
///     AccountInfo { account: String },
/// }
/// ```
///
/// You can compose multiple queries using `#[query_responses(nested)]`. This might be useful
/// together with `#[serde(untagged)]`. If the `nested` flag is set, no `returns` attributes
/// are necessary on the enum variants. Instead, the response types are collected from the
/// nested enums.
///
/// ```
/// # use cosmwasm_schema::QueryResponses;
/// # use schemars::JsonSchema;
/// #[derive(JsonSchema, QueryResponses)]
/// #[query_responses(nested)]
/// #[serde(untagged)]
/// enum QueryMsg {
///     MsgA(QueryA),
///     MsgB(QueryB),
/// }
///
/// #[derive(JsonSchema, QueryResponses)]
/// enum QueryA {
///     #[returns(Vec<String>)]
///     Denoms {},
/// }
///
/// #[derive(JsonSchema, QueryResponses)]
/// enum QueryB {
///     #[returns(AccountInfo)]
///     AccountInfo { account: String },
/// }
///
/// # #[derive(JsonSchema)]
/// # struct AccountInfo {
/// #     IcqHandle: String,
/// # }
/// ```
pub trait QueryResponses: JsonSchema {
    fn response_schemas() -> Result<BTreeMap<String, RootSchema>, IntegrityError> {
        let response_schemas = Self::response_schemas_impl();

        let queries: BTreeSet<_> = response_schemas.keys().cloned().collect();

        check_api_integrity::<Self>(queries)?;

        Ok(response_schemas)
    }

    fn response_schemas_impl() -> BTreeMap<String, RootSchema>;
}

/// Combines multiple response schemas into one. Panics if there are name collisions.
/// Used internally in the implementation of [`QueryResponses`] when using `#[query_responses(nested)]`
pub fn combine_subqueries<const N: usize, T>(
    subqueries: [BTreeMap<String, RootSchema>; N],
) -> BTreeMap<String, RootSchema> {
    let sub_count = subqueries.iter().flatten().count();
    let map: BTreeMap<_, _> = subqueries.into_iter().flatten().collect();
    if map.len() != sub_count {
        panic!(
            "name collision in subqueries for {}",
            std::any::type_name::<T>()
        )
    }
    map
}

/// Returns possible enum variants from `one_of` analysis
fn enum_variants(
    subschemas: SubschemaValidation,
) -> Result<impl Iterator<Item = Result<String, IntegrityError>>, IntegrityError> {
    let iter = subschemas
        .one_of
        .ok_or(IntegrityError::InvalidQueryMsgSchema)?
        .into_iter()
        .map(|s| {
            let s = s.into_object();

            if let Some(SingleOrVec::Single(ty)) = s.instance_type {
                match *ty {
                    // We'll have an object if the Rust enum variant was C-like or tuple-like
                    InstanceType::Object => s
                        .object
                        .ok_or(IntegrityError::InvalidQueryMsgSchema)?
                        .required
                        .into_iter()
                        .next()
                        .ok_or(IntegrityError::InvalidQueryMsgSchema),
                    // We might have a string here if the Rust enum variant was unit-like
                    InstanceType::String => {
                        let values = s.enum_values.ok_or(IntegrityError::InvalidQueryMsgSchema)?;

                        if values.len() != 1 {
                            return Err(IntegrityError::InvalidQueryMsgSchema);
                        }

                        values[0]
                            .as_str()
                            .map(String::from)
                            .ok_or(IntegrityError::InvalidQueryMsgSchema)
                    }
                    _ => Err(IntegrityError::InvalidQueryMsgSchema),
                }
            } else {
                Err(IntegrityError::InvalidQueryMsgSchema)
            }
        });

    Ok(iter)
}

fn verify_queries(
    query_msg: BTreeSet<String>,
    responses: BTreeSet<String>,
) -> Result<(), IntegrityError> {
    if query_msg != responses {
        return Err(IntegrityError::InconsistentQueries {
            query_msg,
            responses,
        });
    }

    Ok(())
}

/// `generated_queries` is expected to be a sorted slice here!
fn check_api_integrity<T: QueryResponses + ?Sized>(
    generated_queries: BTreeSet<String>,
) -> Result<(), IntegrityError> {
    let schema = crate::schema_for!(T);

    let subschemas = if let Some(subschemas) = schema.schema.subschemas {
        subschemas
    } else {
        // No subschemas - no resposnes are expected
        return verify_queries(BTreeSet::new(), generated_queries);
    };

    let schema_queries = if let Some(any_of) = subschemas.any_of {
        // If `any_of` exists, we assume schema is generated from untagged enum
        any_of
            .into_iter()
            .map(|schema| schema.into_object())
            .filter_map(|obj| {
                if let Some(reference) = obj.reference {
                    // Subschemas can be hidden behind references - we want to map them to proper
                    // subschemas in such case

                    // Only references to definitions are supported
                    let reference = match reference.strip_prefix("#/definitions/") {
                        Some(reference) => reference,
                        None => {
                            return Some(Err(IntegrityError::ExternalReference {
                                reference: reference.to_owned(),
                            }))
                        }
                    };

                    let schema = match schema.definitions.get(reference) {
                        Some(schema) => schema.clone(),
                        None => return Some(Err(IntegrityError::InvalidQueryMsgSchema)),
                    };

                    Ok(schema.into_object().subschemas).transpose()
                } else {
                    Ok(obj.subschemas).transpose()
                }
            })
            .map(|subschema| enum_variants(*subschema?))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Result<_, _>>()?
    } else {
        // If `any_of` is not present, there was no untagged enum on top, we expect normal enum at
        // this point
        enum_variants(*subschemas)?.collect::<Result<_, _>>()?
    };

    verify_queries(schema_queries, generated_queries)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IntegrityError {
    #[error("the structure of the QueryMsg schema was unexpected")]
    InvalidQueryMsgSchema,
    #[error("external reference in schema found, but they are not supported")]
    ExternalReference { reference: String },
    #[error(
        "inconsistent queries - QueryMsg schema has {query_msg:?}, but query responses have {responses:?}"
    )]
    InconsistentQueries {
        query_msg: BTreeSet<String>,
        responses: BTreeSet<String>,
    },
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;

    use super::*;

    #[derive(Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    #[allow(dead_code)]
    pub enum GoodMsg {
        BalanceFor { account: String },
        AccountIdFor(String),
        Supply {},
        Liquidity,
        AccountCount(),
    }

    impl QueryResponses for GoodMsg {
        fn response_schemas_impl() -> BTreeMap<String, RootSchema> {
            BTreeMap::from([
                ("balance_for".to_string(), schema_for!(u128)),
                ("account_id_for".to_string(), schema_for!(u128)),
                ("supply".to_string(), schema_for!(u128)),
                ("liquidity".to_string(), schema_for!(u128)),
                ("account_count".to_string(), schema_for!(u128)),
            ])
        }
    }

    #[test]
    fn good_msg_works() {
        let response_schemas = GoodMsg::response_schemas().unwrap();
        assert_eq!(
            response_schemas,
            BTreeMap::from([
                ("balance_for".to_string(), schema_for!(u128)),
                ("account_id_for".to_string(), schema_for!(u128)),
                ("supply".to_string(), schema_for!(u128)),
                ("liquidity".to_string(), schema_for!(u128)),
                ("account_count".to_string(), schema_for!(u128))
            ])
        );
    }

    #[derive(Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    #[allow(dead_code)]
    pub enum EmptyMsg {}

    impl QueryResponses for EmptyMsg {
        fn response_schemas_impl() -> BTreeMap<String, RootSchema> {
            BTreeMap::from([])
        }
    }

    #[test]
    fn empty_msg_works() {
        let response_schemas = EmptyMsg::response_schemas().unwrap();
        assert_eq!(response_schemas, BTreeMap::from([]));
    }

    #[derive(Debug, JsonSchema)]
    #[serde(rename_all = "kebab-case")]
    #[allow(dead_code)]
    pub enum BadMsg {
        BalanceFor { account: String },
    }

    impl QueryResponses for BadMsg {
        fn response_schemas_impl() -> BTreeMap<String, RootSchema> {
            BTreeMap::from([("balance_for".to_string(), schema_for!(u128))])
        }
    }

    #[test]
    fn bad_msg_fails() {
        let err = BadMsg::response_schemas().unwrap_err();
        assert_eq!(
            err,
            IntegrityError::InconsistentQueries {
                query_msg: BTreeSet::from(["balance-for".to_string()]),
                responses: BTreeSet::from(["balance_for".to_string()])
            }
        );
    }

    #[derive(Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    #[allow(dead_code)]
    pub enum ExtMsg {
        Extension {},
    }

    #[derive(Debug, JsonSchema)]
    #[serde(untagged, rename_all = "snake_case")]
    #[allow(dead_code)]
    pub enum UntaggedMsg {
        Good(GoodMsg),
        Ext(ExtMsg),
        Empty(EmptyMsg),
    }

    impl QueryResponses for UntaggedMsg {
        fn response_schemas_impl() -> BTreeMap<String, RootSchema> {
            BTreeMap::from([
                ("balance_for".to_string(), schema_for!(u128)),
                ("account_id_for".to_string(), schema_for!(u128)),
                ("supply".to_string(), schema_for!(u128)),
                ("liquidity".to_string(), schema_for!(u128)),
                ("account_count".to_string(), schema_for!(u128)),
                ("extension".to_string(), schema_for!(())),
            ])
        }
    }

    #[test]
    fn untagged_msg_works() {
        let response_schemas = UntaggedMsg::response_schemas().unwrap();
        assert_eq!(
            response_schemas,
            BTreeMap::from([
                ("balance_for".to_string(), schema_for!(u128)),
                ("account_id_for".to_string(), schema_for!(u128)),
                ("supply".to_string(), schema_for!(u128)),
                ("liquidity".to_string(), schema_for!(u128)),
                ("account_count".to_string(), schema_for!(u128)),
                ("extension".to_string(), schema_for!(())),
            ])
        );
    }
}
