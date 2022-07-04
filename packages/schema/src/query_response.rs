use std::collections::BTreeMap;

use schemars::{schema::RootSchema, JsonSchema};
use thiserror::Error;

pub use cosmwasm_schema_derive::QueryResponses;

pub trait QueryResponses: JsonSchema {
    fn response_schemas() -> Result<BTreeMap<String, RootSchema>, IntegrityError> {
        let response_schemas = Self::response_schemas_impl();

        let queries: Vec<_> = response_schemas
            .keys()
            .map(std::borrow::Borrow::borrow)
            .collect();

        check_api_integrity::<Self>(&queries)?;

        Ok(response_schemas)
    }

    fn response_schemas_impl() -> BTreeMap<String, RootSchema>;
}

/// `generated_queries` is expected to be a sorted slice here!
fn check_api_integrity<T: QueryResponses + ?Sized>(
    generated_queries: &[&str],
) -> Result<(), IntegrityError> {
    let schema = crate::schema_for!(T);

    // something more readable below?

    let mut schema_queries: Vec<_> = schema
        .schema
        .subschemas
        .ok_or(IntegrityError::InvalidQueryMsgSchema)?
        .one_of
        .ok_or(IntegrityError::InvalidQueryMsgSchema)?
        .into_iter()
        .map(|s| {
            s.into_object()
                .object
                .ok_or(IntegrityError::InvalidQueryMsgSchema)?
                .required
                .into_iter()
                .next()
                .ok_or(IntegrityError::InvalidQueryMsgSchema)
        })
        .collect::<Result<_, _>>()?;
    schema_queries.sort();
    if schema_queries != generated_queries {
        return Err(IntegrityError::InconsistentQueries {
            query_msg: schema_queries,
            responses: generated_queries.iter().map(ToString::to_string).collect(),
        });
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("the structure of the QueryMsg schema was unexpected")]
    InvalidQueryMsgSchema,
    #[error(
        "inconsistent queries - QueryMsg schema has {query_msg:?}, but query responses have {responses:?}"
    )]
    InconsistentQueries {
        query_msg: Vec<String>,
        responses: Vec<String>,
    },
}
