use crate::binary::Binary;
use crate::errors::StdError;

pub type QueryResponse = Binary;

pub type QueryResult = Result<QueryResponse, StdError>;
