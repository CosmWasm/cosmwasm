use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::IbcResult;

/// This is the final result type that is created and serialized in a contract for
/// every init/execute/migrate call. The VM then deserializes this type to distinguish
/// between successful and failed executions.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the ComsWasm VM needs to create the same JSON representation.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use cosmwasm_std::{to_vec, IbcContractResult, Response};
/// let response: Response = Response::default();
/// let result: IbcContractResult<Response> = IbcContractResult::Ok(response);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#);
/// ```
///
/// Failure:
///
/// ```
/// # use cosmwasm_std::{to_vec, IbcContractResult, Response};
/// let error_msg = String::from("Something went wrong");
/// let result: IbcContractResult<Response> = IbcContractResult::Err(error_msg);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"error":"Something went wrong"}"#);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcContractResult<S> {
    Ok(S),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
    Abort,
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl<S> IbcContractResult<S> {
    pub fn unwrap(self) -> S {
        match self {
            IbcContractResult::Ok(value) => value,
            IbcContractResult::Err(_) => panic!("error"),
            IbcContractResult::Abort => panic!("abort"),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, IbcContractResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, IbcContractResult::Err(_))
    }
}

// impl<S: fmt::Debug> IbcContractResult<S> {
//     pub fn unwrap_err(self) -> String {
//         self.into_result().unwrap_err()
//         match self {
//             IbcContractResult::Ok(_) => value,
//             IbcContractResult::Err(_) => panic!("error"),
//             IbcContractResult::Abort => panic!("abort"),
//         }
//     }
// }

impl<S, E: ToString> From<Result<S, E>> for IbcContractResult<S> {
    fn from(original: Result<S, E>) -> IbcContractResult<S> {
        match original {
            Ok(value) => IbcContractResult::Ok(value),
            Err(err) => IbcContractResult::Err(err.to_string()),
        }
    }
}

impl<S, E: ToString> From<IbcResult<S, E>> for IbcContractResult<S> {
    fn from(original: IbcResult<S, E>) -> IbcContractResult<S> {
        match original {
            IbcResult::Ok(value) => IbcContractResult::Ok(value),
            IbcResult::Err(err) => IbcContractResult::Err(err.to_string()),
            IbcResult::Abort => IbcContractResult::Abort,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec, Response, StdError, StdResult};

    #[test]
    fn contract_result_serialization_works() {
        let result = IbcContractResult::Ok(12);
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":12}");

        let result = IbcContractResult::Ok("foo");
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":\"foo\"}");

        let result: IbcContractResult<Response> = IbcContractResult::Ok(Response::default());
        assert_eq!(
            to_vec(&result).unwrap(),
            br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#
        );

        let result: IbcContractResult<Response> = IbcContractResult::Err("broken".to_string());
        assert_eq!(&to_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn contract_result_deserialization_works() {
        let result: IbcContractResult<u64> = from_slice(br#"{"ok":12}"#).unwrap();
        assert_eq!(result, IbcContractResult::Ok(12));

        let result: IbcContractResult<String> = from_slice(br#"{"ok":"foo"}"#).unwrap();
        assert_eq!(result, IbcContractResult::Ok("foo".to_string()));

        let result: IbcContractResult<Response> =
            from_slice(br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#)
                .unwrap();
        assert_eq!(result, IbcContractResult::Ok(Response::default()));

        let result: IbcContractResult<Response> = from_slice(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, IbcContractResult::Err("broken".to_string()));

        // ignores whitespace
        let result: IbcContractResult<u64> = from_slice(b" {\n\t  \"ok\": 5898\n}  ").unwrap();
        assert_eq!(result, IbcContractResult::Ok(5898));

        // fails for additional attributes
        let parse: StdResult<IbcContractResult<u64>> =
            from_slice(br#"{"unrelated":321,"ok":4554}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<IbcContractResult<u64>> =
            from_slice(br#"{"ok":4554,"unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<IbcContractResult<u64>> =
            from_slice(br#"{"ok":4554,"error":"What's up now?"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_from_core_result() {
        let original: Result<Response, StdError> = Ok(Response::default());
        let converted: IbcContractResult<Response> = original.into();
        assert_eq!(converted, IbcContractResult::Ok(Response::default()));

        let original: Result<Response, StdError> = Err(StdError::generic_err("broken"));
        let converted: IbcContractResult<Response> = original.into();
        assert_eq!(
            converted,
            IbcContractResult::Err("Generic error: broken".to_string())
        );
    }

    // #[test]
    // fn can_convert_to_core_result() {
    //     let original = IbcContractResult::Ok(Response::default());
    //     let converted: Result<Response, String> = original.into();
    //     assert_eq!(converted, Ok(Response::default()));
    //
    //     let original = IbcContractResult::Err("went wrong".to_string());
    //     let converted: Result<Response, String> = original.into();
    //     assert_eq!(converted, Err("went wrong".to_string()));
    // }
}
