use std::str::from_utf8;

use snafu::ResultExt;

use crate::errors::{Result, Utf8Err};
use crate::storage::Storage;
use crate::types::{Model, QueryResponse, RawQuery};

pub fn perform_raw_query<T: Storage>(store: &T, query: RawQuery) -> Result<QueryResponse> {
    let data = store.get(query.key.as_bytes());
    let results = match data {
        None => vec![],
        Some(val) => {
            let val = from_utf8(&val).context(Utf8Err {})?.to_string();
            vec![Model {
                key: query.key,
                val,
            }]
        }
    };
    Ok(QueryResponse { results })
}
