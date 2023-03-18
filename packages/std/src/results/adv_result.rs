use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This is like Result but we add an Abort case
pub enum AdvResult<S, E> {
    Ok(S),
    Err(E),
    Abort,
}

impl<S, E> From<Result<S, E>> for AdvResult<S, E> {
    fn from(original: Result<S, E>) -> AdvResult<S, E> {
        match original {
            Ok(value) => AdvResult::Ok(value),
            Err(err) => AdvResult::Err(err),
        }
    }
}

/// This is like ContractResult, but we add one more case.
/// This is only used for ibc-receive-packet-adv
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdvContractResult<S> {
    Ok(S),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
    /// This will abort the transaction rather than be managed somewhere
    Abort {},
}

impl<S> AdvContractResult<S> {
    pub fn unwrap(self) -> S {
        match self {
            AdvContractResult::Ok(s) => s,
            AdvContractResult::Err(s) => panic!("{}", s),
            AdvContractResult::Abort {} => panic!("{}", "abort"),
        }
    }
}

impl<S, E: ToString> From<AdvResult<S, E>> for AdvContractResult<S> {
    fn from(original: AdvResult<S, E>) -> AdvContractResult<S> {
        match original {
            AdvResult::Ok(value) => AdvContractResult::Ok(value),
            AdvResult::Err(err) => AdvContractResult::Err(err.to_string()),
            AdvResult::Abort => AdvContractResult::Abort {},
        }
    }
}

impl<S, E: ToString> From<Result<S, E>> for AdvContractResult<S> {
    fn from(original: Result<S, E>) -> AdvContractResult<S> {
        let adv: AdvResult<S, E> = original.into();
        adv.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec, Response, StdError, StdResult};

    #[test]
    fn adv_result_serialization_works() {
        let result = AdvContractResult::Ok(12);
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":12}");

        let result = AdvContractResult::Ok("foo");
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":\"foo\"}");

        let result: AdvContractResult<()> = AdvContractResult::Abort {};
        assert_eq!(&to_vec(&result).unwrap(), b"{\"abort\":{}}");

        let result: AdvContractResult<Response> = AdvContractResult::Ok(Response::default());
        assert_eq!(
            to_vec(&result).unwrap(),
            br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#
        );

        let result: AdvContractResult<Response> = AdvContractResult::Err("broken".to_string());
        assert_eq!(&to_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn adv_result_deserialization_works() {
        let result: AdvContractResult<u64> = from_slice(br#"{"ok":12}"#).unwrap();
        assert_eq!(result, AdvContractResult::Ok(12));

        let result: AdvContractResult<String> = from_slice(br#"{"ok":"foo"}"#).unwrap();
        assert_eq!(result, AdvContractResult::Ok("foo".to_string()));

        let result: AdvContractResult<String> = from_slice(br#"{"abort":{}}"#).unwrap();
        assert_eq!(result, AdvContractResult::Abort {});

        let result: AdvContractResult<Response> =
            from_slice(br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#)
                .unwrap();
        assert_eq!(result, AdvContractResult::Ok(Response::default()));

        let result: AdvContractResult<Response> = from_slice(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, AdvContractResult::Err("broken".to_string()));

        // ignores whitespace
        let result: AdvContractResult<u64> = from_slice(b" {\n\t  \"ok\": 5898\n}  ").unwrap();
        assert_eq!(result, AdvContractResult::Ok(5898));

        // fails for additional attributes
        let parse: StdResult<AdvContractResult<u64>> =
            from_slice(br#"{"unrelated":321,"ok":4554}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<AdvContractResult<u64>> =
            from_slice(br#"{"ok":4554,"unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<AdvContractResult<u64>> =
            from_slice(br#"{"ok":4554,"error":"What's up now?"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_from_core_result() {
        let original: Result<Response, StdError> = Ok(Response::default());
        let converted: AdvContractResult<Response> = original.into();
        assert_eq!(converted, AdvContractResult::Ok(Response::default()));

        let original: Result<Response, StdError> = Err(StdError::generic_err("broken"));
        let converted: AdvContractResult<Response> = original.into();
        assert_eq!(
            converted,
            AdvContractResult::Err("Generic error: broken".to_string())
        );
    }

    #[test]
    fn can_convert_from_adv_result() {
        let original: AdvResult<Response, StdError> = AdvResult::Ok(Response::default());
        let converted: AdvContractResult<Response> = original.into();
        assert_eq!(converted, AdvContractResult::Ok(Response::default()));

        let original: AdvResult<Response, StdError> =
            AdvResult::Err(StdError::generic_err("broken"));
        let converted: AdvContractResult<Response> = original.into();
        assert_eq!(
            converted,
            AdvContractResult::Err("Generic error: broken".to_string())
        );

        let original: AdvResult<Response, StdError> = AdvResult::Abort;
        let converted: AdvContractResult<Response> = original.into();
        assert_eq!(converted, AdvContractResult::Abort {});
    }
}
