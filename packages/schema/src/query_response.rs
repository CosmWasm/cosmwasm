use std::collections::{BTreeMap, BTreeSet};

use schemars::{schema::RootSchema, JsonSchema};
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
