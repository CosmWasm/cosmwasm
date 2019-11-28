use std::str::from_utf8;
use std::vec::Vec;

use snafu::ResultExt;

use crate::errors::{Result, Utf8Err};
use crate::types::{Model, QueryResponse, RawQuery};

pub trait Storage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: &[u8]);
}

pub fn perform_raw_query<T: Storage>(store: &mut T, query: RawQuery) -> Result<QueryResponse> {
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
