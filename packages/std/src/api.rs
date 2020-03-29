/// This maintains types needed for a public API
/// In particular managing serializing and deserializing errors through API boundaries
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::errors::ApiError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiResult<T, E: std::error::Error = ApiError> {
    Ok(T),
    Err(E),
}

impl<T, E: std::error::Error> ApiResult<T, E> {
    pub fn result<U: From<T>>(self) -> Result<U, E> {
        match self {
            ApiResult::Ok(t) => Ok(t.into()),
            ApiResult::Err(e) => Err(e),
        }
    }
}

impl<T, U: From<T>, E: std::error::Error> Into<Result<U, E>> for ApiResult<T, E> {
    fn into(self) -> Result<U, E> {
        self.result()
    }
}

impl<T, U: Into<T>, E: std::error::Error, F: Into<E>> From<Result<U, F>> for ApiResult<T, E> {
    fn from(res: Result<U, F>) -> Self {
        match res {
            Ok(t) => ApiResult::Ok(t.into()),
            Err(e) => ApiResult::Err(e.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::{contract_err, Result};

    #[test]
    fn convert_ok_result() {
        let input: Result<Vec<u8>> = Ok(b"foo".to_vec());
        let convert: ApiResult<Vec<u8>> = input.into();
        assert_eq!(convert, ApiResult::Ok(b"foo".to_vec()));
    }

    #[test]
    fn convert_err_result() {
        let input: Result<()> = contract_err("sample error");
        let convert: ApiResult<()> = input.into();
        assert_eq!(
            convert,
            ApiResult::Err(ApiError::ContractErr {
                msg: "sample error".to_string()
            })
        );
    }
}
