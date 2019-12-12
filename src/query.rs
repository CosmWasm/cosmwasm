use crate::errors::Result;
use crate::traits::Storage;
use crate::types::{Model, QueryResponse, RawQuery};

pub fn perform_raw_query<T: Storage>(store: &T, query: RawQuery) -> Result<QueryResponse> {
    let data = store.get(query.key.as_bytes());
    let results = match data {
        None => vec![],
        Some(val) => vec![Model {
            key: query.key,
            val,
        }],
    };
    Ok(QueryResponse { results })
}
