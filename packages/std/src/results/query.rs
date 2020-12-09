use crate::binary::Binary;
use crate::errors::StdError;

pub type QueryResponse = Binary;

#[deprecated(
    since = "0.12.1",
    note = "QueryResult is deprecated because it uses StdError, which should be replaced with custom errors in CosmWasm 0.11+. \
            Replace this with Result<QueryResponse, StdError> and consider migrating to custom errors from there."
)]
pub type QueryResult = Result<QueryResponse, StdError>;
